# frozen_string_literal: true

Gem::Specification.new do |spec|
  spec.name = 'bernard_ledit'
  spec.version = '0.1.0'
  spec.authors = ['Mindee']
  spec.email = ['opensource@mindee.com']
  spec.summary = "Ruby bindings for Bernard l'Édit: PDF and image utilities."
  spec.homepage = 'https://github.com/mindee/bernard-ledit'
  spec.license = 'MIT'
  # mindee-api-ruby supports Ruby 3.2.
  spec.required_ruby_version = '>= 3.2'

  # lib/ holds the generated bernard_ledit.rb and the native library: `just ruby generate`.
  spec.files = Dir['lib/**/*']
  spec.require_paths = ['lib']

  spec.add_dependency 'ffi', '~> 1.17'
  spec.metadata['rubygems_mfa_required'] = 'true'
end
