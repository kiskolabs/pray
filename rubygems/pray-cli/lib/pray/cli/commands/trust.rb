# frozen_string_literal: true

module Pray
  module CLI
    def trust_list_command
      policy = Trust.load_policy_or_default
      puts "Trust policy (#{Trust.trust_policy_path})"
      puts Trust.format_policy(policy)
    end

    def trust_show_command(source_url)
      policy = Trust.load_policy_or_default
      if source_url.nil? || source_url.empty?
        trust_list_command
        return
      end

      rule = Trust.best_rule(policy, source_url)
      puts "Trust rule for #{source_url}"
      puts "match_prefix: #{rule.match_prefix || "default"}"
      puts "allow: #{rule.allow}"
      puts "allowed_host_keys: #{format_list(rule.allowed_host_keys)}"
      puts "allowed_publishers: #{format_list(rule.allowed_publishers)}"
    end
  end
end
