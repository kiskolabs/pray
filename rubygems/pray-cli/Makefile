.PHONY: release lint test clean

release:
	bundle exec rake release

lint:
	bundle exec rubocop
	bundle exec rbs -I sig validate

test: lint
	bundle exec polyrun parallel-rspec --workers 5 --merge-failures

clean:
	rm -rf coverage .pray/cache tmp pkg
	rm -f spec/examples.txt *.gem .rspec_status
