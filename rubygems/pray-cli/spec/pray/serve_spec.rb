# frozen_string_literal: true

require "spec_helper"
require "fileutils"

RSpec.describe Pray::Serve do
  let(:workspace) { Dir.mktmpdir("pray-serve-") }

  after do
    FileUtils.rm_rf(workspace)
  end

  describe ".dispatch_request" do
    let(:root) { File.join(workspace, "dist") }

    before do
      FileUtils.mkdir_p(File.join(root, "v1"))
      File.write(File.join(root, "v1", "index.json"), "{}")
    end

    it "serves files inside the distribution root" do
      response = described_class.dispatch_request(root, "GET", "/v1/index.json")
      expect(response).to include("200 OK")
      expect(response).to include("{}")
    end

    it "rejects sibling paths that share a prefix with the root" do
      sibling = File.join(workspace, "dist-private")
      FileUtils.mkdir_p(sibling)
      File.write(File.join(sibling, "secret.txt"), "secret")

      response = described_class.dispatch_request(root, "GET", "/../dist-private/secret.txt")
      expect(response).to include("404 Not Found")
    end

    it "rejects paths outside the distribution root" do
      outside = File.join(workspace, "outside")
      FileUtils.mkdir_p(outside)
      File.write(File.join(outside, "secret.txt"), "secret")

      response = described_class.dispatch_request(root, "GET", "/../../outside/secret.txt")
      expect(response).to include("404 Not Found")
    end
  end

  describe ".service_unavailable" do
    it "returns a 503 response body" do
      response = described_class.service_unavailable
      expect(response).to include("503 Service Unavailable")
      expect(response).to include("too many connections")
    end
  end
end
