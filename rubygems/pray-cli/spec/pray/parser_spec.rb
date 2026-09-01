# frozen_string_literal: true

require "spec_helper"

RSpec.describe "Pray parser" do
  it "rejects project paths that escape the repository" do
    expect do
      Pray.parse_manifest(<<~PRAYFILE)
        prayfile "1"
        compose "../outside.md" do
        end
      PRAYFILE
    end.to raise_error(Pray::Error, /escapes repository root/)
  end

  it "parses minimal manifest example" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      source "default", "https://agents.example.com"
      target :tool_a do
        output "INSTRUCTIONS.md"
        skills ".agents/skills"
      end
      agent "sample/base", "~> 1.4",
        exports: ["testing-basics", "security-basics"]
      local ".agents/project.md"
      render mode: :managed,
        conflict: :fail,
        churn: :minimal
    PRAYFILE

    expect(manifest.prayfile_version).to eq("1")
    expect(manifest.sources.first.name).to eq("default")
    expect(manifest.targets.first.name).to eq("tool_a")
    expect(manifest.targets.first.outputs).to eq(["INSTRUCTIONS.md"])
    expect(manifest.packages.first.name).to eq("sample/base")
    expect(manifest.local.first.path).to eq(".agents/project.md")
    expect(manifest.render.mode).to eq("managed")
  end

  it "parses minimal package spec example" do
    package = Pray.parse_package_spec(<<~SPEC)
      Package::Specification.new do |spec|
        spec.name = "sample/base"
        spec.version = "1.4.3"
        spec.summary = "shared guidance"
        spec.files = ["README.md", "exports/testing-basics.md"]
        spec.exports = {
          "testing-basics" => {
            type: "fragment",
            path: "exports/testing-basics.md",
            summary: "Testing guidance"
          }
        }
        spec.add_dependency "sample/common", "~> 1.0"
      end
    SPEC

    expect(package.name).to eq("sample/base")
    expect(package.version).to eq("1.4.3")
    expect(package.files).to eq(["README.md", "exports/testing-basics.md"])
    expect(package.exports["testing-basics"].path).to eq("exports/testing-basics.md")
    expect(package.dependencies.first.name).to eq("sample/common")
  end

  it "preserves package declaration order" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      agent "sample/zebra", "~> 1.0"
      agent "sample/alpha", "~> 1.0"
      agent "sample/middle", "~> 1.0"
    PRAYFILE

    expect(manifest.packages.map(&:name)).to eq(%w[sample/zebra sample/alpha sample/middle])
  end

  it "parses git source keyword form" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      source "amkisko", git: "https://github.com/amkisko/prayers"
      agent "amkisko/working-rules", "~> 1.0", source: "amkisko"
    PRAYFILE

    expect(manifest.sources.length).to eq(1)
    expect(manifest.sources.first.name).to eq("amkisko")
    expect(manifest.sources.first.kind).to eq("git")
    expect(manifest.sources.first.url).to eq("git+https://github.com/amkisko/prayers")
  end

  it "parses git source subdir keyword" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      source "dist", git: "https://github.com/example/prayers", subdir: "prayers"
    PRAYFILE

    expect(manifest.sources.first.subdir).to eq("prayers")
  end

  it "parses git source distribution alias" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      source "amkisko", git: "https://github.com/amkisko/prayers", distribution: "prayers/v1"
    PRAYFILE

    expect(manifest.sources.first.subdir).to eq("prayers/v1")
  end

  it "parses git source rev and tag" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      source "pinned", git: "https://github.com/example/prayers", rev: "abc123def456"
      source "tagged", git: "https://github.com/example/prayers", tag: "v1.0.0"
    PRAYFILE

    expect(manifest.sources[0].rev).to eq("abc123def456")
    expect(manifest.sources[0].tag).to be_nil
    expect(manifest.sources[1].tag).to eq("v1.0.0")
    expect(manifest.sources[1].rev).to be_nil
  end

  it "treats bare package version as exact pin" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      agent "sample/base", "1.0.0"
    PRAYFILE

    expect(manifest.packages.first.constraint).to eq("=1.0.0")
  end

  it "round-trips package declaration through formatter" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      agent "sample/base", "~> 1.0", source: "amkisko", exports: ["testing-basics", "security-basics"]
    PRAYFILE

    formatted = Pray.format_package_declaration(manifest.packages.first)
    expect(formatted).to eq(
      'pray "sample/base", "~> 1.0", source: "amkisko", exports: ["testing-basics", "security-basics"]'
    )
    reparsed = Pray.parse_manifest("prayfile \"1\"\n#{formatted}\n")
    expect(reparsed.packages.first).to eq(manifest.packages.first)
  end

  it "rewrites every matching declaration and keeps indent" do
    text = <<~PRAYFILE
      prayfile "1"
      compose "AGENTS.md" do
        pray "sample/base", "~> 1.0"
      end
      tree ".agents/skills" do
        pray "sample/base", "~> 1.0", export: "testing-basics"
      end
    PRAYFILE
    package = Pray::ManifestPackage.new(name: "sample/base", constraint: "~> 1.1")
    updated = Pray.replace_package_declaration(text, package)
    expect(updated).to include('  pray "sample/base", "~> 1.1"')
    expect(updated).to include('  pray "sample/base", "~> 1.1", export: "testing-basics"')
    expect(updated).not_to include("~> 1.0")
  end

  it "parses pray ssh source url" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      source "team", "pray+ssh://pray@prayers.internal:2222/var/lib/pray"
      agent "sample/base", "1.0.0", source: :team
    PRAYFILE

    expect(manifest.sources.length).to eq(1)
    expect(manifest.sources.first.name).to eq("team")
    expect(manifest.sources.first.kind).to eq("pray_ssh")
    expect(manifest.sources.first.url).to eq("pray+ssh://pray@prayers.internal:2222/var/lib/pray")
  end

  it "rejects manifest without prayfile version" do
    expect do
      Pray.parse_manifest(<<~PRAYFILE)
        target :tool_a do
          output "INSTRUCTIONS.md"
        end
      PRAYFILE
    end.to raise_error(Pray::Error) { |error|
      expect(error.category).to eq(:manifest)
      expect(error.message).to include("missing prayfile version")
    }
  end

  it "rejects package spec without end" do
    expect do
      Pray.parse_package_spec(<<~SPEC)
        Package::Specification.new do |spec|
          spec.name = "sample/base"
      SPEC
    end.to raise_error(Pray::Error) { |error|
      expect(error.category).to eq(:parse)
      expect(error.message).to include("missing 'end'")
    }
  end

  it "matches Rust manifest hash for simple-project" do
    prayfile = File.read(File.expand_path("../../../../examples/simple-project/Prayfile", __dir__))
    manifest = Pray.parse_manifest(prayfile)
    expect(manifest.manifest_hash).to eq(
      "sha256:88e048f95c0a5ec3f09f11d24826f393fc541aebdf0aa50da45fab61d852226c"
    )
  end

  it "parses group blocks and attaches groups to packages" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      group "development", "test" do
        agent "sample/dev", "~> 1.0"
      end
      agent "sample/base", "~> 1.0"
    PRAYFILE

    expect(manifest.packages.map(&:name)).to eq(%w[sample/dev sample/base])
    expect(manifest.packages[0].groups).to eq(%w[development test])
    expect(manifest.packages[1].groups).to eq([])
  end

  it "rejects nested group blocks" do
    expect do
      Pray.parse_manifest(<<~PRAYFILE)
        prayfile "1"
        group "development" do
          group "test" do
            agent "sample/dev", "~> 1.0"
          end
        end
      PRAYFILE
    end.to raise_error(Pray::Error, /nested group blocks are not supported/)
  end

  it "rejects non-package statements inside group blocks" do
    expect do
      Pray.parse_manifest(<<~PRAYFILE)
        prayfile "1"
        group "development" do
          source "default", "https://agents.example.com"
        end
      PRAYFILE
    end.to raise_error(Pray::Error, /group blocks only support agent, package, or pray declarations/)
  end

  it "parses compose blocks with pray and local entries" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      compose "AGENTS.md" do
        pray ".agents/project.md"
        pray "sample/rules", "~> 1.0", path: "packages/rules"
      end
    PRAYFILE

    expect(manifest.targets[0].mode).to eq("compose")
    expect(manifest.targets[0].scoped).to eq(true)
    expect(manifest.targets[0].outputs).to eq(["AGENTS.md"])
    expect(manifest.targets[0].entries.map { |entry| [entry.kind, entry.name || entry.path] }).to eq(
      [["local", ".agents/project.md"], ["package", "sample/rules"]]
    )
    expect(manifest.local[0].bound).to eq(true)
    expect(manifest.packages[0].bound).to eq(true)
    expect(manifest.packages[0].roles).to eq(["fragment"])
  end

  it "parses tree blocks scoping packages to a provisioned folder" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      tree ".agents/skills" do
        pray "sample/audit", "~> 1.0", path: "packages/audit"
      end
    PRAYFILE

    expect(manifest.targets[0].mode).to eq("tree")
    expect(manifest.targets[0].scoped).to eq(true)
    expect(manifest.targets[0].skills).to eq([".agents/skills"])
    expect(manifest.packages[0].bound).to eq(true)
    expect(manifest.packages[0].roles).to eq(["folder"])
  end

  it "parses file: on a pray declaration for exact bindings" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      pray "sample/security", "~> 1.0", path: "packages/security", file: "SECURITY.md"
    PRAYFILE

    expect(manifest.packages[0].file).to eq("SECURITY.md")
    expect(manifest.packages[0].roles).to eq(["file"])
    expect(manifest.packages[0].bound).to eq(true)
  end

  it "parses a file block with a single pray declaration" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      file "SECURITY.md" do
        pray "sample/security", "~> 1.0", path: "packages/security"
      end
    PRAYFILE

    expect(manifest.packages[0].file).to eq("SECURITY.md")
    expect(manifest.packages[0].bound).to eq(true)
  end

  it "rejects a file block without a pray declaration" do
    expect do
      Pray.parse_manifest(<<~PRAYFILE)
        prayfile "1"
        file "SECURITY.md" do
        end
      PRAYFILE
    end.to raise_error(Pray::Error, /requires a pray package declaration/)
  end

  it "parses pray symbol block" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      pray do
        support_email "contact@kiskolabs.com"
        security_email "security@kiskolabs.com"
      end
      pray "sample/base", "~> 1.0"
    PRAYFILE

    expect(manifest.symbols["support_email"]).to eq("contact@kiskolabs.com")
    expect(manifest.symbols["security_email"]).to eq("security@kiskolabs.com")
    expect(manifest.packages.first.name).to eq("sample/base")
  end

  it "parses ruby surface brace blocks and call parentheses for symbols" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      pray{support_email("contact@kiskolabs.com");security_email("security@kiskolabs.com")}
    PRAYFILE

    expect(manifest.symbols["support_email"]).to eq("contact@kiskolabs.com")
    expect(manifest.symbols["security_email"]).to eq("security@kiskolabs.com")
  end

  it "parses ruby surface semicolon do/end one-liner" do
    manifest = Pray.parse_manifest(<<~PRAYFILE)
      prayfile "1"
      pray do; support_email("a@example.com"); security_email("b@example.com"); end
    PRAYFILE

    expect(manifest.symbols["support_email"]).to eq("a@example.com")
    expect(manifest.symbols["security_email"]).to eq("b@example.com")
  end

  it "rejects unimplemented render conflict policy" do
    expect do
      Pray.parse_manifest(<<~PRAYFILE)
        prayfile "1"
        render conflict: :warn
        target :tool_a do
          output "INSTRUCTIONS.md"
        end
      PRAYFILE
    end.to raise_error(Pray::Error, /only :fail is supported/)
  end

  it "rejects duplicate pray symbols" do
    expect do
      Pray.parse_manifest(<<~PRAYFILE)
        prayfile "1"
        pray do
          support_email "a@example.com"
          support_email "b@example.com"
        end
      PRAYFILE
    end.to raise_error(Pray::Error, /duplicate pray symbol/)
  end
end
