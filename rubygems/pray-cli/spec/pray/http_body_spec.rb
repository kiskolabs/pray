# frozen_string_literal: true

require "spec_helper"

RSpec.describe Pray::HttpBody do
  it "accepts chunks within the ceiling and rejects oversized content" do
    body = +"".b
    described_class.append_chunk!(body, "ok".b, 8)
    expect(body).to eq("ok".b)

    expect do
      described_class.append_chunk!(body, "too-large".b, 8)
    end.to raise_error(Pray::Error, /HTTP response exceeds/)

    expect do
      described_class.reject_oversized_content_length!(64, 8)
    end.to raise_error(Pray::Error, /HTTP response exceeds/)

    response = chunked_response(["ok".b], content_length: 2)
    expect(described_class.read_response!(response, 8)).to eq("ok".b)
  end

  def chunked_response(chunks, content_length: nil)
    response = Object.new
    response.define_singleton_method(:[]) do |name|
      (name == "content-length") ? content_length&.to_s : nil
    end
    response.define_singleton_method(:read_body) do |&block|
      chunks.each { |chunk| block.call(chunk) }
    end
    response
  end
end
