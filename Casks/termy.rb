cask "termy" do
  arch arm: "arm64", intel: "x86_64"

  version "0.1.30"
  sha256 arm:   "a5a960742321c14434e35e94adddea16bbebfe5aa4c1e4fcbc92aa5d5ede1dd0",
         intel: "76655abd330b7b45b3e34082317ecbd72b3951c127f312ef04c05ec4cad335d0"

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
