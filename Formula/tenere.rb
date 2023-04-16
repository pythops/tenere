class Tenere < Formula
  desc "TUI interface for LLMs built in Rust"
  homepage "https://github.com/pythops/tenere"
  license "AGPLv3"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/pythops/tenere/releases/download/v0.1/tenere-aarch64-macos"
      sha256 "83a5c5b5f5d6c18a1a2f1d5a6e5a244e5da6d2b1c9b7e343d6c55810c0bdc36e"
    elsif Hardware::CPU.intel?
      url "https://github.com/pythops/tenere/releases/download/v0.1/tenere-x86_64-macos"
      sha256 "d8082a2f72513a15d47c95aa40e26f69b8cc34b30d29de52f1d60c02184ed7a0"
    end
  end

  def install
    bin.install "tenere-aarch64-macos" => "tenere" if Hardware::CPU.arm?
    bin.install "tenere-x86_64-macos" => "tenere" if Hardware::CPU.intel?
  end
end
