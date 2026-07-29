# frozen_string_literal: true

require "json"
require_relative "session"

module Pray
  ConfessionSubmission = Struct.new(
    :package, :version, :status, :note, :lockfile,
    :distribution_point, :signer, :timestamp, :signature,
    keyword_init: true
  )

  module Confess
    module_function

    def submit(package:, from_lock:, version:, accepted:, rejected:, note:, url:,
      project_root: Dir.pwd)
      raise Error.unsupported("confess requires exactly one of --accepted or --rejected") if accepted == rejected

      manifest_path = File.join(project_root, "Prayfile")
      project = Resolve.resolve_project(manifest_path)
      lockfile_path = File.join(project_root, "Prayfile.lock")
      lockfile = File.file?(lockfile_path) ? Lockfile.read_lockfile(lockfile_path) : nil

      package_name, resolved_version, package_resolution =
        resolve_target(project, lockfile, package, from_lock, version)
      source_url = resolve_source_url(project, package_resolution, package_name, url)
      if source_url.start_with?("pray+ssh://", "ssh+pray://")
        raise Error.unsupported("pray_ssh confession submit is not implemented yet in pray-cli Ruby")
      end

      confession = ConfessionSubmission.new(
        package: package_name,
        version: resolved_version,
        status: accepted ? "accepted" : "rejected",
        note: note,
        lockfile: lockfile&.file_hash,
        distribution_point: source_url,
        signer: Session.current_signer(project_root),
        timestamp: Time.now.to_i.to_s,
        signature: nil
      )
      confession.signature = Hashing.sha256_prefixed(JSON.generate(confession_to_hash(confession)))
      Registry.http_post(
        Registry.join_url(source_url, "v1/confessions"),
        "application/json",
        JSON.generate(confession_to_hash(confession))
      )
      puts "Confession submitted for #{confession.package} #{confession.version}"
    end

    def resolve_target(project, lockfile, package, from_lock, version)
      if from_lock
        raise Error.resolution("confess --from-lock requires an existing lockfile") unless lockfile

        span = lockfile.managed_span.find { |record| record.id == from_lock }
        raise Error.resolution("lockfile span #{from_lock} not found") unless span

        package_resolution = project.packages.find { |entry| entry.declaration.name == span.package }
        raise Error.resolution("package #{span.package} not found") unless package_resolution

        locked = lockfile.package.find { |entry| entry.name == span.package }
        raise Error.resolution("lockfile package #{span.package} not found") unless locked

        if version && version != locked.version
          raise Error.resolution(
            "lockfile span #{from_lock} version #{locked.version} does not match requested version #{version}"
          )
        end
        [span.package, version || locked.version, package_resolution]
      else
        raise Error.unsupported("confess requires a package name") unless package

        package_resolution = project.packages.find { |entry| entry.declaration.name == package }
        raise Error.resolution("package #{package} not found") unless package_resolution

        if version && version != package_resolution.spec.version
          raise Error.resolution(
            "package #{package} version #{version} does not match resolved version #{package_resolution.spec.version}"
          )
        end
        [package, version || package_resolution.spec.version, package_resolution]
      end
    end

    def resolve_source_url(project, package_resolution, package_name, url)
      return url if url

      source_name = package_resolution.declaration.source
      raise Error.resolution("package #{package_name} is missing a source") unless source_name

      source = project.manifest.sources.find { |entry| entry.name == source_name }
      raise Error.resolution("unknown source: #{source_name}") unless source

      source.url
    end

    def confession_to_hash(confession)
      hash = {
        "package" => confession.package,
        "version" => confession.version,
        "status" => confession.status
      }
      hash["note"] = confession.note if confession.note
      hash["lockfile"] = confession.lockfile if confession.lockfile
      hash["distribution_point"] = confession.distribution_point if confession.distribution_point
      hash["signer"] = confession.signer if confession.signer
      hash["timestamp"] = confession.timestamp if confession.timestamp
      hash["signature"] = confession.signature if confession.signature
      hash
    end
  end
end
