#!/usr/bin/env ruby
# frozen_string_literal: true

require 'json'
require 'net/http'
require 'open3'
require 'optparse'
require 'uri'

options = {
  client_paths: []
}

OptionParser.new do |parser|
  parser.on('--pray-bin PATH') { |value| options[:pray_bin] = value }
  parser.on('--server-url URL') { |value| options[:server_url] = value }
  parser.on('--client PATH', 'Client workspace path', proc { |value| options[:client_paths] << value })
end.parse!

if options[:pray_bin].to_s.empty?
  abort '--pray-bin is required'
end
if options[:server_url].to_s.empty?
  abort '--server-url is required'
end
if options[:client_paths].length < 2
  abort 'at least two --client paths are required'
end

pray_bin = options[:pray_bin]
server_url = options[:server_url].sub(%r{/$}, '')
client_a, client_b = options[:client_paths].first(2)


def run_pray(pray_bin, arguments, directory)
  stdout, stderr, status = Open3.capture3(pray_bin, *arguments, chdir: directory)
  unless status.success?
    raise "pray #{arguments.join(' ')} failed in #{directory}\nSTDOUT:\n#{stdout}\nSTDERR:\n#{stderr}"
  end
  [stdout, stderr]
end

def fetch_html(url)
  uri = URI(url)
  response = Net::HTTP.get_response(uri)
  unless response.is_a?(Net::HTTPSuccess)
    raise "GET #{url} failed with #{response.code}"
  end
  response.body
end

def assert_includes_text(html, text)
  return if html.include?(text)

  raise "missing #{text.inspect}"
end

def assert_includes_link(html, label, href)
  return if html.include?(href) && html.include?(label)

  raise "missing link #{label.inspect} to #{href.inspect}"
end

run_pray(pray_bin, ['install'], client_a)
run_pray(pray_bin, ['install'], client_b)

run_pray(pray_bin, ['confess', 'sample/base', '--accepted', '--note', 'client A found the package useful'], client_a)
run_pray(pray_bin, ['confess', 'sample/base', '--rejected', '--note', 'client B needs a narrower checklist'], client_b)

root_page = fetch_html("#{server_url}/")
assert_includes_text(root_page, 'Pray distribution point')
assert_includes_link(root_page, 'sample/base', '/packages/sample/base')

package_page = fetch_html("#{server_url}/packages/sample/base")
assert_includes_text(package_page, 'sample/base')
assert_includes_text(package_page, 'Accepted: 1')
assert_includes_text(package_page, 'Rejected: 1')
assert_includes_text(package_page, 'Signer: sample-agent-packages@example.com')
assert_includes_text(package_page, 'Signature: sha256:')
assert_includes_text(package_page, 'client A found the package useful')
assert_includes_text(package_page, 'client B needs a narrower checklist')
assert_includes_link(
  package_page,
  '1.4.3',
  '/v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg'
)

puts 'distribution point smoke test passed'
