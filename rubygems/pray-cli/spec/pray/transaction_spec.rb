# frozen_string_literal: true

require "spec_helper"

RSpec.describe "project write recovery" do
  it "rolls back failures and recovers crashes while preserving later edits" do
    Dir.mktmpdir("pray-transaction-") do |root|
      path = File.join(root, "Prayfile")
      File.write(path, "original")
      expect do
        Pray::Transaction.run(root) do
          Pray::Transaction.write_file(path, "candidate")
          Pray::Transaction.write_file(File.join(root, "output"), "created")
          raise "late failure"
        end
      end.to raise_error(/late failure/)
      expect(File.read(path)).to eq("original")
      expect(File.exist?(File.join(root, "output"))).to be(false)
      child = fork do
        Pray::Transaction.run(root) do
          Pray::Transaction.write_file(path, "intermediate")
          Pray::Transaction.write_file(path, "candidate")
          Pray::Transaction.write_file(File.join(root, "output"), "created")
          exit! 91
        end
      end
      Process.wait(child)
      expect($?.exitstatus).to eq(91)
      File.write(path, "operator edit")
      expect { Pray::Transaction.run(root) {} }.to raise_error(Pray::Error, /changed/)
      expect(File.read(path)).to eq("operator edit")
      File.rename(path, File.join(root, "operator.saved"))
      Pray::Transaction.run(root) {}
      expect(File.read(path)).to eq("original")
    end
  end

  it "excludes a second writer until commit" do
    Dir.mktmpdir("pray-transaction-concurrent-") do |root|
      Pray::Transaction.run(root) do
        Pray::Transaction.write_file(File.join(root, "output"), "first")
        Thread.new do
          expect { Pray::Transaction.run(root) { Pray::Transaction.write_file(File.join(root, "output"), "second") } }
            .to raise_error(Pray::Error, /another pray command/)
        end.value
      end
      expect(File.read(File.join(root, "output"))).to eq("first")
    end
  end

  it "retains recovery when restored bytes still need a failed directory sync" do
    Dir.mktmpdir("pray-transaction-sync-") do |root|
      parent = File.join(root, "output")
      FileUtils.mkdir_p(parent)
      path = File.join(parent, "file")
      File.write(path, "original")
      fail_sync = false
      allow(Pray::TransactionOwner).to receive(:sync_directory).and_wrap_original do |original, directory|
        raise Errno::EIO if fail_sync && directory == parent

        original.call(directory)
      end
      expect do
        Pray::Transaction.run(root) do
          Pray::Transaction.write_file(path, "candidate")
          fail_sync = true
          raise "late failure"
        end
      end.to raise_error(Pray::Error, /Recovery stopped/)
      expect(File.read(path)).to eq("original")
      expect { Pray::Transaction.run(root) { raise "must not start" } }.to raise_error(Errno::EIO)
      fail_sync = false
      Pray::Transaction.run(root) {}
      expect(File.read(path)).to eq("original")
    end
  end
end
