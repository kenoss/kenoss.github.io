ZOLA_VERSION := "0.21.0"

check-version:
  [ "$(zola --version)" == "zola {{ZOLA_VERSION}}" ] || (zola --version; false)

build:
  just check-version && zola build

serve:
  just check-version && zola serve
