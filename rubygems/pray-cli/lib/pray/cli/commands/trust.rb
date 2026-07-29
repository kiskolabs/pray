# frozen_string_literal: true

module Pray
  module CLI
    def trust_list_command(scope: :all, source_url: nil)
      puts Trust.list_policy(scope: scope, source_url: source_url)
    end

    def trust_show_command
      puts Trust.show_policy_toml
    end

    def trust_add_key_command(key, match_prefix: nil)
      Trust.add_allowed_signing_key(key, match_prefix: match_prefix)
    end

    def trust_remove_key_command(key, match_prefix: nil)
      Trust.remove_allowed_signing_key(key, match_prefix: match_prefix)
    end

    def trust_set_signed_command(match_prefix:, enabled:)
      Trust.set_require_signed_commit(match_prefix, enabled)
    end

    def trust_set_allow_command(match_prefix:, allow:)
      Trust.set_allow(match_prefix, allow)
    end

    def trust_import_repo_command(source_url, match_prefix: nil)
      clone_url = source_url.delete_prefix("git+")
      repository = GitSources.git_source_cache_directory(Dir.pwd, clone_url)
      unless File.directory?(File.join(repository, ".git"))
        raise Error.resolution(
          "no cached git repository for #{clone_url} at #{repository}"
        )
      end

      added = Trust.import_repo(
        clone_url, repository, match_prefix: match_prefix || clone_url
      )
      puts "imported #{added} key(s) from #{repository}"
    end

    def trust_import_registry_command(source_url, match_prefix: nil, include_host_key: false)
      result = Trust.import_registry(
        source_url, match_prefix: match_prefix, include_host_key: include_host_key
      )
      puts "imported #{result[:publishers_added]} publisher fingerprint(s) and " \
           "#{result[:host_keys_added]} host key(s) for #{match_prefix || source_url}"
    end

    def trust_check_command(source = nil)
      source_description, hits = Trust.check_compromised(source)
      if hits.empty?
        puts "no compromised trusted signing keys detected (checked against #{source_description})"
        return
      end

      hits.each do |key, scopes, matches|
        puts "[compromised] #{key}"
        puts "  scopes: #{scopes.join(", ")}"
        matches.each do |entry|
          puts "  reason: #{entry.reason}" if entry.reason
          puts "  reference: #{entry.reference}" if entry.reference
          puts "  reported_at: #{entry.reported_at}" if entry.reported_at
        end
      end
      raise Error.integrity(
        "found #{hits.length} compromised trusted key(s) in #{source_description}"
      )
    end
  end
end
