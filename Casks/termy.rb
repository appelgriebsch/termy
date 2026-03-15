cask "termy" do
  arch arm: "arm64", intel: "x86_64"

  version "0.1.52"
  sha256 arm:   "62d3f3f58d3867ffce878f877b6d71d25c88f1dfd2a875ac355b22e6b0e6d320",
         intel: "5060cbb94195143e356480dd9f14b2c6d8872c7b4f22168b8263a950d1794122"

  url "https://github.com/lassejlv/termy/releases/download/v#{version}/Termy-v#{version}-macos-#{arch}.dmg"
  name "Termy"
  desc "Minimal GPU-powered terminal written in Rust"
  homepage "https://github.com/lassejlv/termy"

  livecheck do
    url :url
    strategy :github_latest
  end

  depends_on macos: ">= :big_sur"

  app "Termy.app"
end
