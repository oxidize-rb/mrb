# frozen_string_literal: true

desc "Format the code"
task :fmt do
  sh "cargo fmt"
  sh "npx prettier --write '**/*.md'"
end

desc "Run the tests"
task :test do
  sh "cargo test"
end