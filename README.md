# vlazba / gimyzba

## Changes
- This is an app rewritten from https://github.com/teleological/vlazba and https://github.com/lynn/vlazba .
- Ported to Rust from Python 3

## Usage

Create a gismu from transliterations:

    vlazba "<Mandarin> <Hindi> <English> <Spanish> <Russian> <Arabic>"

For example:

    vlazba "uan rakan ekspekt esper predpologa mulud"

Use `-w` to specify a list of weights. The default is `0.347,0.196,0.160,0.123,0.089,0.085` and corresponds to the six languages above. The official Lojban gismu were made with this configuration. Here is a weighted 8-language configuration by Ilmen:

    vlazba -w 0.271,0.170,0.130,0.125,0.104,0.076,0.064,0.060  mandarin english spanish hindi arabic bengali russian portuguese

See old-README.txt for more info.
