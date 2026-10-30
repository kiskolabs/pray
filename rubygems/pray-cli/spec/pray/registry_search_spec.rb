# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require "open3"

RSpec.describe Pray::RegistrySearch do
  let(:root) { Dir.mktmpdir("pray-registry-search-") }

  after { FileUtils.rm_rf(root) }

  it "matches names case-insensitively and reads the latest non-yanked summary" do
    write_index(%w[sample/base sample/webapp])
    write_metadata("sample/base", [
      version("1.0.0", "old summary"),
      version("2.0.0", "ignored summary", yanked: true),
      version("1.1.0", "current summary")
    ])

    hits = described_class.search_local_registry(root, "BASE", include_summary: true)

    expect(hits.map(&:name)).to eq(["sample/base"])
    expect(hits.first.summary).to eq("current summary")
  end

  it "rejects unsafe package names from an untrusted index" do
    write_index(["../outside"])

    expect do
      described_class.search_local_registry(root, "", include_summary: true)
    end.to raise_error(Pray::Error, /invalid package name/)
  end

  it "requires namespace and name segments in untrusted index entries" do
    ["unscoped", "sample/extra/name"].each do |name|
      write_index([name])

      expect do
        described_class.search_local_registry(root, "", include_summary: false)
      end.to raise_error(Pray::Error, /invalid package name/)
    end
  end

  it "rejects non-string package names from an untrusted index" do
    write_index([7])

    expect do
      described_class.search_local_registry(root, "", include_summary: false)
    end.to raise_error(Pray::Error, /package names must be strings/)
  end

  it "exposes search through the CLI without declaring the package" do
    write_index(["sample/base"])
    write_metadata("sample/base", [version("1.0.0", "shared guidance")])
    executable = File.expand_path("../../bin/pray", __dir__)

    stdout, stderr, status = Open3.capture3("ruby", executable, "search", "base", "--root", root)

    expect(status).to be_success, stderr
    expect(stdout).to eq("sample/base — shared guidance\n")
  end

  it "searches an HTTP index and includes available summaries" do
    index = JSON.generate("spec" => "prayfile-distribution-1", "packages" => ["sample/base"])
    metadata = JSON.generate("name" => "sample/base", "versions" => [version("1.0.0", "shared guidance")])
    allow(Pray::Registry).to receive(:http_get).with("https://registry.example/v1/index.json").and_return(index)
    allow(Pray::Registry).to receive(:http_get)
      .with("https://registry.example/v1/packages/sample/base.json").and_return(metadata)

    hits = described_class.search_http_registry("https://registry.example", "base")

    expect(hits.map { |hit| [hit.name, hit.summary] }).to eq([["sample/base", "shared guidance"]])
  end

  def write_index(packages)
    FileUtils.mkdir_p(File.join(root, "v1", "packages"))
    File.write(
      File.join(root, "v1", "index.json"),
      JSON.generate("spec" => "prayfile-distribution-1", "packages" => packages)
    )
  end

  def write_metadata(name, versions)
    path = File.join(root, "v1", "packages", "#{name}.json")
    FileUtils.mkdir_p(File.dirname(path))
    File.write(path, JSON.generate("name" => name, "versions" => versions))
  end

  def version(number, summary, yanked: false)
    {
      "version" => number,
      "artifact" => "v1/artifacts/sample/base/#{number}/package.praypkg",
      "yanked" => yanked,
      "derived_metadata" => {"summary" => summary}
    }
  end
end
