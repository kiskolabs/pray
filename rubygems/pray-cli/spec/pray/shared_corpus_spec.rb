# frozen_string_literal: true

require "spec_helper"
require "json"

RSpec.describe "shared fixture corpus" do
  corpus_root = File.expand_path("../../../../testdata/shared/manifest", __dir__)

  Dir.children(corpus_root)
    .select { |name| File.directory?(File.join(corpus_root, name)) }
    .sort
    .each do |case_name|
      it "parses #{case_name} against expected.json" do
        dir = File.join(corpus_root, case_name)
        text = File.read(File.join(dir, "Prayfile"))
        expected = JSON.parse(File.read(File.join(dir, "expected.json")))
        manifest = Pray.parse_manifest(text)

        expect(manifest.targets.length).to eq(expected["targets"].length)
        expected["targets"].each_with_index do |want, index|
          target = manifest.targets[index]
          expect(target.name).to eq(want["name"])
          expect(target.mode).to eq(want["mode"])
          expect(target.scoped).to eq(want["scoped"])
          expect(target.outputs).to eq(want["outputs"] || [])
          expect(target.skills).to eq(want["skills"] || [])
          expect(target.entries.map { |entry|
            fields = {"kind" => entry.kind}
            fields["name"] = entry.name if entry.name
            fields["path"] = entry.path if entry.path
            fields
          }).to eq(want["entries"])
        end

        expect(manifest.packages.length).to eq(expected["packages"].length)
        expected["packages"].each_with_index do |want, index|
          package = manifest.packages[index]
          expect(package.name).to eq(want["name"])
          expect(package.bound).to eq(want["bound"])
          expect(package.roles).to eq(want["roles"])
          expect(package.file).to eq(want["file"])
          expect(package.path).to eq(want["path"])
        end

        expect(manifest.local.length).to eq(expected["local"].length)
        expected["local"].each_with_index do |want, index|
          local = manifest.local[index]
          expect(local.path).to eq(want["path"])
          expect(local.bound).to eq(want["bound"])
        end
      end
    end
end
