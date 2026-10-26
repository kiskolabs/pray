# frozen_string_literal: true

# CLI entry. Loads the core library plus Pray::CLI.
# Embed consumers that do not need the CLI should require "pray" instead.

require_relative "pray"
require_relative "pray/cli"
