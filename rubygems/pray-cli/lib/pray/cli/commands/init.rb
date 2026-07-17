# frozen_string_literal: true

require "json"
require "fileutils"

module Pray
  module CLI
    def manifest_command
      manifest = Pray.parse_manifest(Pray.read_manifest_text(manifest_path))
      puts JSON.pretty_generate(ManifestJson.manifest_fields(manifest.canonicalized))
    end

    def init_command(targets)
      path = manifest_path
      raise Error.manifest("Prayfile already exists") if File.exist?(path)

      targets = ["tool_a"] if targets.empty?
      lines = ['prayfile "1"']
      targets.each do |target|
        output = default_output_for_target(target)
        lines << "target :#{target} do"
        lines << "  output \"#{output}.md\""
        lines << "end"
      end
      File.write(path, "#{lines.join("\n")}\n")
    end

    def prayer_init_command
      root = Dir.pwd
      package_name = File.basename(root)
      package_name = "prayer-package" if package_name.strip.empty?
      prayspec_path = File.join(root, "#{package_name}.prayspec")
      raise Error.manifest("package spec already exists: #{prayspec_path}") if File.exist?(prayspec_path)

      File.write(
        prayspec_path,
        <<~SPEC
          Package::Specification.new do |spec|
            spec.name = "#{package_name}"
            spec.version = "0.1.0"
            spec.summary = "Prayer package"
            spec.files = []
          end
        SPEC
      )
    end

    def repo_init_command
      distribution_root = repo_distribution_root(Dir.pwd)
      index_path = File.join(distribution_root, "v1", "index.json")
      raise Error.manifest("distribution repo already exists") if File.exist?(index_path)

      FileUtils.mkdir_p(File.join(distribution_root, "v1", "packages"))
      FileUtils.mkdir_p(File.join(distribution_root, "v1", "artifacts"))
      Publish.write_registry_index(distribution_root, RegistryIndex.new)
    end

    def repo_distribution_root(root)
      (File.basename(root) == "prayers") ? root : File.join(root, "prayers")
    end
  end
end
