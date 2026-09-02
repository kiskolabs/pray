# frozen_string_literal: true

module Pray
  module ComposeDest
    module_function

    HTML_COMMENT_EXTENSIONS = %w[.md .markdown .html .htm].freeze
    BINARY_EXTENSIONS = %w[
      .png .jpg .jpeg .gif .webp .ico .pdf .zip .gz .tgz .tar .wasm .bin
      .woff .woff2 .exe .dylib .so
    ].freeze

    def compose_writes_header?(target, output, project_header)
      return target.header unless target.header.nil?

      project_header && agents_markdown?(output)
    end

    def header_text(target, output, project_header)
      return unless compose_writes_header?(target, output, project_header)

      name = File.basename(output)
      guidance = if agents_markdown?(output)
        "Do not edit managed blocks in `#{name}` or provisioned files under `.agents/`."
      else
        "Do not edit managed blocks in `#{name}`."
      end
      <<~HEADER.rstrip
        <!-- pray:0 ignore-comments -->

        # Agent context

        #{guidance}
        To change shared guidance, update `Prayfile` and run `pray install`.
      HEADER
    end

    def ensure_html_comment_dest!(output)
      dest = output.tr("\\", "/")
      extension = File.extname(output).downcase
      return if HTML_COMMENT_EXTENSIONS.include?(extension)
      if extension == ".json"
        raise Error.render("compose cannot write JSON; use file: \"#{dest}\" for unmarked bytes")
      end
      if BINARY_EXTENSIONS.include?(extension)
        raise Error.render(
          "compose cannot write a binary file; use file: \"#{dest}\" for unmarked bytes"
        )
      end
      raise Error.render(
        "compose cannot write this file type; use file: \"#{dest}\" for unmarked bytes"
      )
    end

    def agents_markdown?(output)
      File.basename(output) == "AGENTS.md"
    end
  end
end
