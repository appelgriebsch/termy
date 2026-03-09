cask "termy" do
  arch arm: "arm64", intel: "x86_64"

  version "0.1.48"
  sha256 arm:   "19a2222b0d2eea8405ec8f0a4dc08762cb893f84b5d962c9fd3456c1a8b70f12",
         intel: "47dac173f33ebebeaaefba1c4b30e9b5efe16f6e4e84be8a655991c8dd5528e5"

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
