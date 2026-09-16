# frozen_string_literal: true

module Pray
  module Render
    module_function

    def append_managed_local(builder, managed_spans, local, target, output, symbols)
      return if local.content.empty? && local.optional

      body = Substitute.substitute_pray_symbols(local.content, symbols)
      append_managed_span(
        builder,
        managed_spans,
        "local:#{local.manifest_path}:#{target.name}",
        body,
        output,
        LOCAL_EMBED_PACKAGE,
        local.manifest_path,
        local.source_checksum
      )
    end

    def append_managed_export(builder, managed_spans, package, export, target, output, symbols)
      body = package.export_bodies[export]
      unless body
        raise Error.integrity("compose cannot write binary export #{export}; use file: for unmarked bytes") if package.spec.exports[export]&.kind == "file"
        raise Error.render("package #{package.declaration.name} is missing cached export #{export}")
      end

      body = Substitute.substitute_pray_symbols(body, symbols)
      append_managed_span(
        builder,
        managed_spans,
        "#{package.declaration.name}:#{export}:#{target.name}",
        body,
        output,
        package.declaration.name,
        export,
        package.source_checksum
      )
    end

    def append_managed_span(builder, managed_spans, seed, body, output, package, export, source_checksum)
      identifier = Hashing.marker_id(seed)
      open_line = builder.next_line_number
      builder.append_line("<!-- pray:#{identifier} -->")
      builder.append_body(body)
      close_line = builder.next_line_number
      builder.append_line("<!-- pray:#{identifier} -->")
      managed_spans << ManagedSpanRecord.new(
        id: identifier,
        target: output,
        open_line: open_line,
        close_line: close_line,
        ideal_checksum: Hashing.checksum_managed_span_content(body),
        package: package,
        export: export,
        source_checksum: source_checksum,
        silenced: false
      )
      builder.append_empty_line
    end
  end
end
