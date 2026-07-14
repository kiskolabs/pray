# frozen_string_literal: true

module Pray
  module CLI
    def publish_command(roots:, servers:)
      project = resolve_current_project
      roots.each { |root| Publish.publish_to_root(project, root) }
      servers.each { |server| Publish.publish_to_server(project, server) }
    end

    def serve_command(root:, host:, port:, stdio:)
      raise Error.unsupported("serve --stdio is not implemented yet in pray-cli Ruby") if stdio

      Serve.run_server(root: root, host: host, port: port)
    end
  end
end
