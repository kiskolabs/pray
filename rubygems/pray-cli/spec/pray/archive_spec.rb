# frozen_string_literal: true

require "spec_helper"
require "fileutils"
require "open3"

RSpec.describe Pray::Archive do
  let(:workspace) { Dir.mktmpdir("pray-archive-") }
  let(:package_root) do
    root = File.join(workspace, "package")
    FileUtils.mkdir_p(root)
    File.write(
      File.join(root, "demo.prayspec"),
      <<~PRAYSPEC
        Package::Specification.new do |spec|
          spec.name = "demo"
          spec.version = "1.0.0"
          spec.files = ["rules.md"]
        end
      PRAYSPEC
    )
    File.write(File.join(root, "rules.md"), "# demo\n")
    root
  end
  let(:spec) { Pray.parse_package_spec(File.read(File.join(package_root, "demo.prayspec"))).canonicalized }
  let(:package) do
    Pray::ResolvedPackage.new(
      declaration: Pray::ManifestPackage.new(name: "demo", constraint: "1.0.0"),
      root: package_root,
      spec: spec,
      tree_hash: spec.tree_hash_for_root(package_root),
      selected_exports: []
    )
  end

  after do
    FileUtils.rm_rf(workspace)
  end

  describe ".unpack_praypkg" do
    it "unpacks when Encoding.default_internal is UTF-8" do
      artifact_bytes = described_class.build_package_archive_bytes(package)
      output_directory = File.join(workspace, "unpacked")
      previous_internal = Encoding.default_internal
      previous_external = Encoding.default_external || Encoding::UTF_8

      Encoding.default_external = Encoding::UTF_8
      Encoding.default_internal = Encoding::UTF_8

      expect do
        described_class.unpack_praypkg(artifact_bytes, output_directory)
      end.not_to raise_error

      expect(File).to exist(File.join(output_directory, "demo.prayspec"))
      expect(File).to exist(File.join(output_directory, "rules.md"))
    ensure
      Encoding.default_internal = previous_internal
      Encoding.default_external = previous_external
    end

    it "rejects parent directory escape paths" do
      tar_bytes = ustar_bytes("../escape.md", "owned\n")
      artifact_bytes = zstd_bytes(tar_bytes)
      output_directory = File.join(workspace, "escape-out")

      expect do
        described_class.unpack_praypkg(artifact_bytes, output_directory)
      end.to raise_error(Pray::Error, /escapes package root/)
    end

    it "rejects tar headers with invalid checksums" do
      tar_bytes = ustar_bytes("rules.md", "ok\n")
      tar_bytes.setbyte(0, tar_bytes.getbyte(0) ^ 0xFF)
      artifact_bytes = zstd_bytes(tar_bytes)

      expect do
        described_class.unpack_praypkg(artifact_bytes, File.join(workspace, "checksum-out"))
      end.to raise_error(Pray::Error, /checksum/)
    end

    it "accepts a checksum field written as seven octal digits" do
      tar_bytes = ustar_bytes("rules.md", "ok\n", checksum_format: "%07o\0")
      artifact_bytes = zstd_bytes(tar_bytes)
      output_directory = File.join(workspace, "seven-digit-checksum-out")

      described_class.unpack_praypkg(artifact_bytes, output_directory)

      expect(File).to exist(File.join(output_directory, "rules.md"))
    end

    it "accepts a checksum field padded with leading spaces" do
      tar_bytes = ustar_bytes("rules.md", "ok\n", checksum_format: "%7o\0")
      artifact_bytes = zstd_bytes(tar_bytes)
      output_directory = File.join(workspace, "padded-checksum-out")

      described_class.unpack_praypkg(artifact_bytes, output_directory)

      expect(File).to exist(File.join(output_directory, "rules.md"))
    end

    it "rejects duplicate archive paths" do
      tar_bytes = ustar_entry("rules.md", "first\n") +
        ustar_entry("rules.md", "second\n") + ("\0" * 1024)
      artifact_bytes = zstd_bytes(tar_bytes)

      expect do
        described_class.unpack_praypkg(artifact_bytes, File.join(workspace, "duplicate-out"))
      end.to raise_error(Pray::Error, /duplicate/)
    end

    it "skips AppleDouble sidecar members" do
      tar_bytes = ustar_entry("demo.prayspec", "ok\n") +
        ustar_entry("._demo.prayspec", "sidecar\n") + ("\0" * 1024)
      artifact_bytes = zstd_bytes(tar_bytes)
      output_directory = File.join(workspace, "appledouble-out")

      described_class.unpack_praypkg(artifact_bytes, output_directory)

      expect(File).to exist(File.join(output_directory, "demo.prayspec"))
      expect(File).not_to exist(File.join(output_directory, "._demo.prayspec"))
    end

    it "rejects oversized compressed artifacts" do
      oversized = ("\0" * (Pray::ResourceLimits::MAX_ARCHIVE_TOTAL_BYTES + 1)).b
      output_directory = File.join(workspace, "oversize-out")

      expect do
        described_class.unpack_praypkg(oversized, output_directory)
      end.to raise_error(Pray::Error, /exceeds/)
    end
  end

  def ustar_bytes(path, content, checksum_format: "%06o\0 ")
    ustar_entry(path, content, checksum_format: checksum_format) + ("\0" * 1024)
  end

  # checksum_format fills the eight byte checksum field. Writers differ: six octal
  # digits then NUL and space, seven digits then NUL, or space padding.
  def ustar_entry(path, content, checksum_format: "%06o\0 ")
    content = content.b
    path = path.b
    header = +"".b
    header << path.ljust(100, "\0")
    header << "0000644\0"
    header << "0000000\0"
    header << "0000000\0"
    header << format("%011o\0", content.bytesize)
    header << "00000000000\0"
    header << "        "
    header << "0"
    header << ("\0" * 100)
    header << "ustar\0"
    header << "00"
    header << ("\0" * 32)
    header << ("\0" * 32)
    header << "0000000\0"
    header << "0000000\0"
    header << ("\0" * 155)
    header = header.ljust(512, "\0")
    sum = header.bytes.sum
    header[148, 8] = format(checksum_format, sum).b
    pad = (512 - (content.bytesize % 512)) % 512
    header + content + ("\0" * pad)
  end

  def zstd_bytes(tar_bytes)
    out, status = Open3.capture2("zstd", "-q", "-c", stdin_data: tar_bytes)
    raise "zstd failed" unless status.success?

    out
  end
end
