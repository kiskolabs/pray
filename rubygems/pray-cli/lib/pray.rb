# frozen_string_literal: true

require_relative "pray/version"
require_relative "pray/error"
require_relative "pray/literal"
require_relative "pray/hashing"
require_relative "pray/constraint"
require_relative "pray/manifest_json"
require_relative "pray/manifest"
require_relative "pray/package_spec"
require_relative "pray/lockfile"
require_relative "pray/config"
require_relative "pray/dotenv"
require_relative "pray/project_context"
require_relative "pray/environment"
require_relative "pray/resolve_context"
require_relative "pray/invocation"
require_relative "pray/resolve"
require_relative "pray/render"
require_relative "pray/verify"
require_relative "pray/registry"
require_relative "pray/archive"
require_relative "pray/plan"
require_relative "pray/git_sources"
require_relative "pray/publish"
require_relative "pray/serve"
require_relative "pray/trust"
require_relative "pray/materialize"

class << Pray
  public :parse_manifest, :read_manifest_text, :format_package_declaration, :replace_package_declaration,
    :parse_package_spec, :parse_lockfile, :read_lockfile, :serialize_lockfile, :lockfile_hash,
    :write_lockfile, :write_lockfile_if_changed, :lockfiles_equivalent?, :build_lockfile,
    :resolve_project, :render_project, :materialize_project,
    :inspect_project, :verify_project, :drift_project,
    :default_manifest_path, :default_lockfile_path, :project_root_from_manifest
end
