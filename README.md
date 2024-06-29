# vlazba / gimyzba / jvozba

A Rust-based tool for generating Lojban gismu (root words) and lujvo (compound words).

## Features

- Generates gismu based on input from transliterations of words in multiple languages
- Creates lujvo using the jvozba algorithm
- Customizable language weighting
- Efficient Rust implementation

## Installation

1. Ensure you have Rust installed and up to date:

   ```bash
   rustup default stable
   rustup update
   ```

2. Clone the repository and build the project:

   ```bash
   git clone https://github.com/your-username/vlazba.git
   cd vlazba
   cargo build --release
   ```

3. The executable will be available at `target/release/vlazba`

## Usage

### Gismu Generation

Basic usage:

```bash
./vlazba "<Mandarin> <Hindi> <English> <Spanish> <Russian> <Arabic>"
```

Example:

```bash
./vlazba "uan rakan ekspekt esper predpologa mulud"
```

Custom weights:

```bash
./vlazba -w 0.271,0.170,0.130,0.125,0.104,0.076,0.064,0.060 mandarin english spanish hindi arabic bengali russian portuguese
```

### Lujvo Creation (jvozba)

To create lujvo using the jvozba algorithm:

```bash
./vlazba --jvozba "<word1> <word2> <word3>"
```

```bash
./vlazba --jvozba --exp-rafsi "<word1> <word2> <word3>"
```

Examples:

```bash
./vlazba --jvozba "klama klama gasnu"
```

```bash
./vlazba --jvozba --exp-rafsi "corci klama gasnu"
```

### Lujvo Decomposition (jvokaha)

To split lujvo using the jvokaha algorithm:

```bash
./vlazba --jvokaha "<lujvo>"
```

```bash
./vlazba --jvokajha --exp-rafsi "<lujvo>"
```

Examples:

```bash
./vlazba --jvozba "klaklagau"
```

```bash
./vlazba --jvozba --exp-rafsi "cocklagau"
```

## Options

- `-w, --weights`: Specify custom language weights (default: 0.347,0.196,0.160,0.123,0.089,0.085)
- `-s, --shapes`: Define gismu candidate shapes (default: "ccvcv,cvccv")
- `-a, --all-letters`: Use all available letters instead of only those in input words
- `-d, --deduplicate`: Path to existing gismu list for deduplication
- `--jvozba`: Use jvozba function to create lujvo instead of gismu generation
- `--forbid-la-lai-doi`: Forbid 'la', 'lai', 'doi' in lujvo when using jvozba
- `--jvokaha`: Use jvokaha function to split lujvo into components
- `--exp-rafsi`: Include experimental rafsi when generating lujvo

## Debug

```bash
RUST_BACKTRACE=full cargo run -- "uan rakan ekspekt esper predpologa mulud"
```

## Background

This project is a Rust rewrite of the original [gimyzba](https://github.com/teleological/gimyzba) and its [Python port](https://github.com/lynn/gimyzba). It aims to provide a more efficient and maintainable implementation of the gismu generation algorithm. Additionally it ports [jvozba](https://github.com/sozysozbot/sozysozbot_jvozba/tree/master) algorithm for getting lujvo creation functionality.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the [GNU GENERAL PUBLIC LICENSE](LICENSE).
