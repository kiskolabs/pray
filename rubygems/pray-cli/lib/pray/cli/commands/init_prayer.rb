# frozen_string_literal: true

require "fileutils"

module Pray
  module CLI
    def prayer_init_command(name = nil, directory = nil)
      if File.exist?(manifest_path)
        scaffold_local_prayer(name, directory)
      else
        scaffold_standalone_package(name)
      end
    end

    def scaffold_local_prayer(name, directory)
      requested = name.to_s.strip
      package_name = LocalPrayer.validate_name!(requested.empty? ? DEFAULT_LOCAL_PRAYER_NAME : requested)
      source_name, source_directory = resolve_local_prayer_directory(directory)
      root = File.join(project_root, source_directory, package_name)
      prayspec_path = File.join(root, "#{package_name}.prayspec")
      raise Error.manifest("package spec already exists: #{prayspec_path}") if File.exist?(prayspec_path)
      raise Error.manifest("prayer directory already exists: #{root}") if File.exist?(root)

      FileUtils.mkdir_p(File.join(root, "exports"))
      File.write(prayspec_path, local_prayspec(source_name, package_name))
      File.write(File.join(root, "README.md"), "# #{package_name}\n")
      File.write(File.join(root, "exports", "#{package_name}.md"), "# #{package_name}\n")
      declare_local_prayer("#{source_name}/#{package_name}")
    end

    def scaffold_standalone_package(name)
      root = Dir.pwd
      package_name = if name.nil? || name.strip.empty?
        basename = File.basename(root)
        basename.strip.empty? ? "prayer-package" : basename
      else
        LocalPrayer.validate_name!(name)
      end
      prayspec_path = File.join(root, "#{package_name}.prayspec")
      raise Error.manifest("package spec already exists: #{prayspec_path}") if File.exist?(prayspec_path)

      File.write(prayspec_path, standalone_prayspec(package_name))
      readme = File.join(root, "README.md")
      File.write(readme, "# #{package_name}\n") unless File.exist?(readme)
      FileUtils.mkdir_p(File.join(root, "exports"))
    end

    def local_prayspec(source_name, name)
      <<~SPEC
        Package::Specification.new do |spec|
          spec.name = "#{source_name}/#{name}"
          spec.summary = "Describe this package"
          spec.files = ["README.md", "exports/#{name}.md"]
          spec.exports = {
            "#{name}" => {
              type: "fragment",
              path: "exports/#{name}.md"
            }
          }
        end
      SPEC
    end

    def standalone_prayspec(name)
      <<~SPEC
        Package::Specification.new do |spec|
          spec.name = "#{name}"
          spec.version = "0.1.0"
          spec.summary = "Describe this package"
          spec.files = ["README.md"]
          spec.exports = {}
        end
      SPEC
    end
  end
end
