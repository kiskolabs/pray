# frozen_string_literal: true

module Pray
  module CLI
    def publish_command(roots:, servers:, to: [], dry_run: false)
      project = resolve_current_project
      dests = PublishSelect.resolve_publish_destinations(
        project.manifest.publish_remotes || [],
        PublishSelect::PublishCliDest.new(to: to, roots: roots, servers: servers),
        Dir.pwd
      )
      if dry_run
        print_publish_plan(project, dests)
        return
      end
      dests.each do |dest|
        packages = selected_publish_packages(project, dest.packages)
        filtered = project.dup
        filtered.packages = packages
        Publish.publish_to_root(filtered, dest.root) if dest.root
        Publish.publish_to_server(filtered, dest.server) if dest.server
      end
    end

    def selected_publish_packages(project, listed)
      allowed = PublishRemote.allowed_publish_names(project.manifest, listed)
      selected = project.packages.select { |package| allowed.include?(package.declaration.name) }
      if selected.empty?
        raise Error.usage("no path packages to publish; remote dependencies are not published")
      end
      selected.each { |package| package.spec.require_release_version! }
      selected
    end

    def print_publish_plan(project, dests)
      dests.each do |dest|
        packages = selected_publish_packages(project, dest.packages)
        target = dest.root || dest.server || dest.name
        packages.each do |package|
          puts "publish #{package.declaration.name} #{package.spec.version} -> #{dest.name} (#{target})"
        end
      end
    end

    def serve_command(root:, host:, port:, stdio:, to: nil)
      raise Error.unsupported("serve --stdio is not implemented yet in pray-cli Ruby") if stdio

      Serve.run_server(root: optional_path_remote_root(to, root), host: host, port: port)
    end

    def optional_path_remote_root(to, root)
      remotes = begin
        Pray.parse_manifest(Pray.read_manifest_text(manifest_path)).publish_remotes || []
      rescue Error
        return root if to.nil?

        raise
      end
      return root if remotes.empty? && to.nil?

      cli_root = (root == ".") ? nil : root
      PublishSelect.resolve_path_remote(remotes, to, cli_root, Dir.pwd)
    end
  end
end
