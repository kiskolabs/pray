# Ruby gems

Ruby packages for the pray project.

## pray-cli

Ruby `pray` CLI. Resolves `Prayfile` dependencies, writes `Prayfile.lock`, renders managed guidance, and verifies drift.

Install from this directory:

```bash
cd rubygems/pray-cli
bundle install
bundle exec pray version
```

Or install as a gem from [RubyGems](https://rubygems.org/gems/pray-cli):

```bash
gem install pray-cli
```

Run tests:

```bash
cd rubygems/pray-cli && bundle exec rspec
```

From the repository root:

```bash
make ruby-test
```

The executable name is `pray`, matching the Rust CLI.
