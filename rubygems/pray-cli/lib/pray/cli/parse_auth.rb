# frozen_string_literal: true

module Pray
  module CLI
    def parse_login_arguments(arguments)
      servers = []
      email = nil
      passkey_key = nil
      credential_id = nil
      ssh_agent = false
      public_key = nil
      while (argument = arguments.shift)
        case argument
        when "--server" then servers << arguments.shift
        when "--email" then email = arguments.shift
        when "--passkey-key" then passkey_key = arguments.shift
        when "--credential-id" then credential_id = arguments.shift
        when "--ssh-agent" then ssh_agent = true
        when "--public-key" then public_key = arguments.shift
        else
          raise Error.unsupported("unknown login argument: #{argument}")
        end
      end
      raise Error.unsupported("login requires --server URL") if servers.empty?
      raise Error.unsupported("login requires --email EMAIL") if email.to_s.empty?

      passkey = passkey_key || credential_id
      if passkey && ssh_agent
        raise Error.unsupported("login accepts either passkey or ssh-agent mode, not both")
      end
      if passkey_key
        raise Error.unsupported("passkey login requires --credential-id") if credential_id.to_s.empty?

        {servers: servers, email: email, mode: :passkey, passkey_key: passkey_key,
         credential_id: credential_id}
      elsif ssh_agent
        raise Error.unsupported("ssh-agent login requires --public-key") if public_key.to_s.empty?

        {servers: servers, email: email, mode: :ssh_agent, public_key: public_key}
      else
        raise Error.unsupported("login requires --passkey-key/--credential-id or --ssh-agent")
      end
    end

    def parse_confess_arguments(arguments)
      package = nil
      from_lock = nil
      version = nil
      accepted = false
      rejected = false
      note = nil
      url = nil
      while (argument = arguments.shift)
        case argument
        when "--from-lock" then from_lock = arguments.shift
        when "--version" then version = arguments.shift
        when "--accepted" then accepted = true
        when "--rejected" then rejected = true
        when "--note" then note = arguments.shift
        when "--url" then url = arguments.shift
        when /\A-/ then raise Error.unsupported("unknown confess argument: #{argument}")
        else package = argument
        end
      end
      if accepted == rejected
        raise Error.unsupported("confess requires exactly one of --accepted or --rejected")
      end
      if package && from_lock
        raise Error.unsupported("confess accepts either a package name or --from-lock")
      end
      raise Error.unsupported("confess requires a package name") if package.nil? && from_lock.nil?

      {package: package, from_lock: from_lock, version: version, accepted: accepted,
       rejected: rejected, note: note, url: url}
    end

    def parse_sync_arguments(arguments)
      root = "."
      peers = []
      while (argument = arguments.shift)
        case argument
        when "--root" then root = arguments.shift
        when "--peer" then peers << arguments.shift
        else
          raise Error.unsupported("unknown sync argument: #{argument}")
        end
      end
      {root: root, peers: peers}
    end
  end
end
