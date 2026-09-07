# frozen_string_literal: true

require "fileutils"
require "json"
require "open3"
require "rbconfig"
require "tmpdir"

repository = File.expand_path("..", __dir__)
rust_test = Dir[File.join(repository, "target/debug/deps/write_transaction-*")]
  .select { |path| File.file?(path) && File.executable?(path) && File.extname(path).empty? }
  .max_by { |path| File.mtime(path) }
abort "build the Rust write_transaction test first" unless rust_test

typescript = File.join(repository, "npmjs/pray-cli/dist/transaction/index.js")
abort "build the TypeScript CLI first" unless File.file?(typescript)

ruby_script = <<~RUBY
  require "pray"
  root, action = ARGV
  Pray::Transaction.run(root) do
    if action == "crash"
      Pray::Transaction.write_file(File.join(root, "Prayfile"), "intermediate")
      Pray::Transaction.write_file(File.join(root, "Prayfile"), "candidate")
      Pray::Transaction.write_file(File.join(root, "output"), "created")
      exit! 91
    end
  end
RUBY
typescript_script = <<~JAVASCRIPT
  import {runTransaction, writeProjectFile} from #{typescript.to_json};
  const [root, action] = process.argv.slice(1);
  runTransaction(root, () => {
    if (action === "crash") {
      writeProjectFile(root + "/Prayfile", "intermediate");
      writeProjectFile(root + "/Prayfile", "candidate");
      writeProjectFile(root + "/output", "created");
      process.exit(91);
    }
  });
JAVASCRIPT

invoke = lambda do |implementation, root, action|
  environment = {"PRAY_TRANSACTION_TEST_ROOT" => root, "PRAY_TRANSACTION_TEST_ACTION" => action}
  command = case implementation
  when "rust" then [rust_test, "--exact", "crash_child", "--nocapture"]
  when "typescript" then ["node", "--input-type=module", "-e", typescript_script, root, action]
  when "ruby"
    [RbConfig.ruby, "-rbundler/setup", "-I", File.join(repository, "rubygems/pray-cli/lib"),
      "-e", ruby_script, root, action]
  end
  output, status = Open3.capture2e(environment, *command)
  [status.exitstatus, output]
end

%w[rust typescript ruby].permutation(2).each do |producer, consumer|
  [false, true].each do |edited|
    Dir.mktmpdir("pray-cross-recovery-") do |root|
      path = File.join(root, "Prayfile")
      File.write(path, "original")
      File.chmod(0o640, path)
      status, output = invoke.call(producer, root, "crash")
      raise "#{producer} failed to leave recovery state: #{output}" unless status == 91

      journal = File.join(root, ".pray/write-state/journal")
      File.open(journal, "ab") { |file| file.write('{"type":"write","entry":') }
      if edited
        File.write(path, "operator edit")
        status, = invoke.call(consumer, root, "recover")
        raise "recovery overwrote an edit" if status == 0 || File.read(path) != "operator edit"

        File.rename(path, File.join(root, "operator.saved"))
      end
      status, output = invoke.call(consumer, root, "recover")
      raise "#{producer} -> #{consumer} recovery failed: #{output}" unless status == 0
      raise "original bytes were lost" unless File.read(path) == "original"
      raise "original permissions were lost" unless File.stat(path).mode & 0o777 == 0o640
      raise "created output survived rollback" if File.exist?(File.join(root, "output"))
      raise "recovery did not finish" if File.exist?(journal)
      if edited && File.read(File.join(root, "operator.saved")) != "operator edit"
        raise "saved edit was lost"
      end

      puts "#{producer} -> #{consumer}, edited=#{edited}: passed"
    end
  end
end
