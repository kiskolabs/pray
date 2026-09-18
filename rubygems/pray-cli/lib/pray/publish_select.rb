# frozen_string_literal: true

module Pray
  module PublishSelect
    module_function

    PublishDestination = Struct.new(:name, :root, :server, :packages) do
      def initialize(name:, root: nil, server: nil, packages: [])
        super
      end
    end
    PublishCliDest = Struct.new(:to, :roots, :servers) do
      def initialize(to: [], roots: [], servers: [])
        super
      end
    end

    def resolve_publish_destinations(remotes, cli, project_root)
      if !cli.to.empty? && (!cli.roots.empty? || !cli.servers.empty?)
        raise Error.usage("publish --to cannot be combined with --root or --server")
      end
      return resolve_undeclared_destinations(cli) if remotes.empty?

      select_remotes(remotes, cli, project_root).map do |remote|
        PublishDestination.new(
          name: remote.name,
          root: remote.path && File.join(project_root, remote.path),
          server: remote.url,
          packages: remote.packages
        )
      end
    end

    def resolve_path_remote(remotes, to, root, project_root)
      path_remotes = remotes.select(&:path)
      if to
        remote = remotes.find { |entry| entry.name == to }
        raise Error.usage("unknown publish remote: #{to}") unless remote
        unless remote.path
          raise Error.usage("publish remote #{to} is a URL; yank, serve, and token need a path remote")
        end
        return File.join(project_root, remote.path)
      end
      if remotes.empty?
        raise Error.usage("requires --root PATH") unless root

        return root
      end
      if root
        matched = remotes.find { |remote| remote.path && root_matches?(root, remote.path, project_root) }
        raise Error.usage("path #{root} is not a declared publish remote") unless matched

        return root
      end
      if path_remotes.length == 1 && path_remotes.first.path
        return File.join(project_root, path_remotes.first.path)
      end

      raise Error.usage("say which path remote with --to NAME or --root PATH")
    end

    def resolve_undeclared_destinations(cli)
      if !cli.to.empty?
        raise Error.usage("Prayfile has no publish remotes; pass --root PATH or --server URL")
      end
      if cli.roots.empty? && cli.servers.empty?
        raise Error.unsupported("publish requires at least one --root PATH or --server URL")
      end

      dests = cli.roots.map { |root| PublishDestination.new(name: root, root: root, packages: []) }
      dests + cli.servers.map { |server| PublishDestination.new(name: server, server: server, packages: []) }
    end

    def select_remotes(remotes, cli, project_root)
      unless cli.to.empty?
        return cli.to.map do |name|
          remote = remotes.find { |entry| entry.name == name }
          raise Error.usage("unknown publish remote: #{name}") unless remote

          remote
        end
      end
      return remotes if cli.roots.empty? && cli.servers.empty?

      selected = cli.roots.map do |root|
        remote = remotes.find { |entry| entry.path && root_matches?(root, entry.path, project_root) }
        raise Error.usage("path #{root} is not a declared publish remote") unless remote

        remote
      end
      selected + cli.servers.map do |server|
        remote = remotes.find { |entry| entry.url == server }
        raise Error.usage("server #{server} is not a declared publish remote") unless remote

        remote
      end
    end

    def root_matches?(cli_root, remote_path, project_root)
      declared = File.join(project_root, remote_path)
      return true if cli_root == declared || cli_root == remote_path

      strip_dot_slash(cli_root) == strip_dot_slash(remote_path)
    end

    def strip_dot_slash(value)
      value.to_s.strip.delete_prefix("./").delete_suffix("/").tr("\\", "/")
    end
  end
end
