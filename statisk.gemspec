# coding: utf-8
lib = File.expand_path('../lib', __FILE__)
$LOAD_PATH.unshift(lib) unless $LOAD_PATH.include?(lib)
require 'statisk/version'

Gem::Specification.new do |spec|
  spec.name          = 'statisk'
  spec.version       = Statisk::VERSION
  spec.authors       = ['Sondre Nilsen']
  spec.email         = ['nilsen.sondre@gmail.com']

  spec.summary       = %q(Stupidly simple static site generator)
  spec.description   = %q(Stupidly simple static site generator with batteries included)
  spec.homepage      = 'https://gitlab.com'
  spec.license       = 'MIT'

  spec.files         = `git ls-files -z`.split("\x0").reject { |f| f.match(%r{^(test|spec|features)/}) }
  spec.executables   = spec.files.grep(%r{^bin/}) { |f| File.basename(f) }
  spec.require_paths = ['lib']

  spec.add_dependency 'colorize', '~> 0.7'
  spec.add_dependency 'highline', '~> 1.7'
  spec.add_dependency 'redcarpet', '~> 3.3'

  spec.add_development_dependency 'bundler', '~> 1.10'
  spec.add_development_dependency 'rake', '~> 10.0'
  spec.add_development_dependency 'minitest', '~> 5.8'
  spec.add_development_dependency 'guard', '~> 2.13'
  spec.add_development_dependency 'guard-minitest', '~> 2.4'
end
