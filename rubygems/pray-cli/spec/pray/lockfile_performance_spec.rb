# frozen_string_literal: true

require "spec_helper"

RSpec.describe "large lockfile parsing" do
  def destination_ledger
    "prayfile_lock = \"1\"\n" + (0...1000).map do |index|
      <<~TOML
        [[provisioned]]
        path = "out/#{index}"
        content_hash = "sha256:#{"a" * 64}"
        package = "sample/files"
        export = "files"
      TOML
    end.join
  end

  it "loads a thousand destinations within a bounded allocation budget" do
    text = destination_ledger
    allocated = GC.stat(:total_allocated_objects)
    lockfile = Pray.parse_lockfile(text)
    used = GC.stat(:total_allocated_objects) - allocated

    expect(lockfile.provisioned.size).to eq(1000)
    expect(lockfile.provisioned.last.path).to eq("out/999")
    expect(used).to be < 60_000
  end

  it "plans from a large previous lock within one parsing allocation budget" do
    original_context = Pray::Invocation.context
    Dir.mktmpdir("pray-large-plan-") do |root|
      File.write(File.join(root, "Prayfile"), "prayfile \"1\"\n")
      File.write(File.join(root, "Prayfile.lock"), destination_ledger)
      allocated = GC.stat(:total_allocated_objects)
      expect { Pray::CLI.run(["--path", root, "plan"]) }.to output(/Lockfile: update/).to_stdout
      used = GC.stat(:total_allocated_objects) - allocated

      expect(used).to be < 60_000
    end
  ensure
    Pray::Invocation.context = original_context
  end

  it "collects normalized tree destinations within a bounded allocation budget" do
    Dir.mktmpdir("pray-large-tree-") do |root|
      names = (0...1000).map { |index| "./#{index}.txt" }
      names.each { |name| File.write(File.join(root, name), "content") }
      project = Pray::ResolvedProject.new(project_root: root)
      destinations = []
      allocated = GC.stat(:total_allocated_objects)
      Pray::Render.collect_tree_files(project, root, File.join(root, "out/./nested"), names,
        [], [], "sample/files", "files", destinations)
      used = GC.stat(:total_allocated_objects) - allocated

      expect(destinations.size).to eq(1000)
      expect(destinations.last.path).to eq("out/nested/999.txt")
      expect(used).to be < 100_000
    end
  end
end
