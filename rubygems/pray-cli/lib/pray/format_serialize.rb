# frozen_string_literal: true

module Pray
  module FormatSerialize
    module_function

    def serialize_recommended(manifest)
      lines = [%(prayfile "#{manifest.prayfile_version}")]

      unless manifest.sources.empty?
        lines << ""
        manifest.sources.each { |source| lines << format_source(source) }
      end

      unless manifest.symbols.empty?
        lines << ""
        lines << "pray do"
        manifest.symbols.sort.each do |key, value|
          lines << %(  #{key} "#{value}")
        end
        lines << "end"
      end

      manifest.targets.each do |target|
        next unless target.scoped

        lines << ""
        case target.mode
        when "compose"
          path = target.outputs.first || ""
          header = case target.header
          when true then ", header: true"
          when false then ", header: false"
          else ""
          end
          lines << %(compose "#{path}"#{header} do)
          target.entries.each do |entry|
            lines << "  #{format_destination_entry(entry, manifest)}"
          end
          lines << "end"
        when "tree"
          path = target.skills.first || ""
          lines << %(tree "#{path}" do)
          target.entries.each do |entry|
            next unless entry.kind == "package"

            package = find_package(manifest, entry.name)
            lines << "  #{ManifestMethods.format_package_declaration(package)}" if package
          end
          lines << "end"
        end
      end

      file_packages = manifest.packages.select(&:file)
      unless file_packages.empty?
        lines << ""
        file_packages.each { |package| lines << ManifestMethods.format_package_declaration(package) }
      end

      unbound = manifest.packages.select { |package| !package.bound && !package.file && package.groups.empty? }
      unless unbound.empty?
        lines << ""
        unbound.each { |package| lines << ManifestMethods.format_package_declaration(package) }
      end

      grouped_packages(manifest).each do |group_names, packages|
        lines << ""
        lines << "group #{group_names.map { |name| ":#{name}" }.join(", ")} do"
        packages.each { |package| lines << "  #{ManifestMethods.format_package_declaration(package)}" }
        lines << "end"
      end

      manifest.targets.each do |target|
        next if target.scoped || !target_has_extras?(target)

        lines << ""
        lines << "target :#{target.name} do"
        target.commands.each { |command| lines << %(  commands "#{command}") }
        target.rules.each { |rule| lines << %(  rules "#{rule}") }
        lines << "  max_bytes #{target.max_bytes}" if target.max_bytes
        lines << "end"
      end

      if manifest.render != RenderPolicy.default
        lines << ""
        lines << "render mode: :#{manifest.render.mode}, conflict: :#{manifest.render.conflict}, " \
                 "churn: :#{manifest.render.churn}"
      end

      lines << ""
      lines.join("\n")
    end

    def target_has_extras?(target)
      !target.commands.empty? || !target.rules.empty? || !target.max_bytes.nil?
    end

    def format_source(source)
      parts = [%(source "#{source.name}")]
      case source.kind
      when "path"
        parts << %(path: "#{source.url}")
      when "git"
        url = source.url.delete_prefix("git+")
        parts << %(git: "#{url}")
      else
        parts << %("#{source.url}")
      end
      parts << %(distribution: "#{source.subdir}") if source.subdir
      parts << %(tag: "#{source.tag}") if source.tag
      parts << %(rev: "#{source.rev}") if source.rev
      parts.join(", ")
    end

    def format_destination_entry(entry, manifest)
      if entry.kind == "local"
        return %(pray "#{entry.path}")
      end

      package = find_package(manifest, entry.name)
      package ? ManifestMethods.format_package_declaration(package) : %(pray "#{entry.name}")
    end

    def find_package(manifest, name)
      manifest.packages.find { |package| package.name == name }
    end

    def grouped_packages(manifest)
      groups = {}
      manifest.packages.each do |package|
        next if package.groups.empty?

        key = package.groups.dup
        groups[key] ||= []
        groups[key] << package
      end
      groups.sort_by { |names, _| names }.map { |names, packages| [names, packages] }
    end
  end
end
