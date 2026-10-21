# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::GitMaterialize do
  describe "recovery hints" do
    it "names git_revision_unfetched when the revision is missing" do
      expect do
        described_class.require_revision_present!("/tmp/pray-missing-git-db", "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef")
      end.to raise_error(Pray::Error) { |error|
        expect(error.message).to include("git_revision_unfetched")
        expect(error.message).to include("pray install --locked")
      }
    end

    it "names offline when revision is missing offline" do
      expect do
        described_class.require_revision_present!(
          "/tmp/pray-missing-git-db",
          "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
          offline: true
        )
      end.to raise_error(Pray::Error) { |error|
        expect(error.message).to include("git_revision_unfetched")
        expect(error.message).to include("offline mode is enabled")
      }
    end

    it "names git_cache_stale on materialize failure" do
      expect do
        described_class.annotate_git_materialize_failure!(
          "/tmp/pray-stale-git-db",
          "abc123",
          "git archive failed"
        )
      end.to raise_error(Pray::Error) { |error|
        expect(error.message).to include("git_cache_stale")
        expect(error.message).to include("pray install --locked")
      }
    end

    it "refuses URL-shaped local catalog artifacts" do
      expect do
        described_class.read_local_artifact_bytes("/tmp/dist", "https://evil.example/pkg.praypkg")
      end.to raise_error(Pray::Error) { |error|
        expect(error.message).to include("relative paths")
        expect(error.message).to include("not URLs")
      }
    end
  end
end
