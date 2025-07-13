# Changelog

All notable changes to the rule72 commit message reflow tool will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2] - 2025-07-10

### Documentation
- **Updated README**: Improved documentation with v0.2.1 features and current performance metrics
- **Performance Metrics**: Updated performance from ~2ms to ~1.5ms based on latest profiling results
- **Debug Features**: Added comprehensive documentation for new `--debug-svg` and `--debug-trace` CLI flags
- **Markdown Formatting**: Improved line wrapping and readability throughout README

## [0.2.1] - 2025-07-10

### Added
- **Debug Tracing**: New `--debug-trace` flag provides comprehensive pipeline visibility with automatic file:line prefixes
- **Automated Trace Prefixes**: Debug traces now include programmatically determined file and line information

### Fixed
- **SVG Double Indentation**: Fixed SVG debug output incorrectly showing extra leading spaces for indented lines
- **Import Cleanup**: Removed unused imports detected by linting

### Technical
- Enhanced lexer with detailed trace output for explainability
- Created debug_trace! macro for consistent logging with automatic source location
- Updated function signatures to support Options parameter throughout the parsing pipeline

## [0.2.0] - 2025-07-08

### Added
- **Debug SVG Visualization**: New `--debug-svg` flag generates SVG files showing document parsing and classification
- **List Introduction Support**: Smart detection and grouping of introduction lines (ending with `:`) with following lists
- **Enhanced Nix Integration**: 
  - Added `nix-test` recipe to Justfile for testing callPackage builds
  - Updated `default.nix` to work properly with callPackage using cargoLock
- **Performance Profiling**: Added `profile` target to Justfile for performance analysis
- **Comprehensive Test Data**: Added synthetic commit message datasets for testing (good, medium, bad examples)

### Changed
- **Improved Footer Classification**: Replaced overly broad regex pattern matching with explicit footer tag detection
- **Better List Handling**: Enhanced list parsing to maintain semantic grouping with introduction lines
- **Refined Document Structure**: Improved spacing logic between headlines, body chunks, and footers
- **Enhanced Unicode Support**: Better handling of Unicode characters in list markers and text measurement

### Fixed
- **Content Truncation Bug**: Fixed issue where lines like "EN: something broke" were incorrectly classified as footers
- **Extra Blank Lines**: Resolved problem of adding unnecessary blank lines after headlines
- **List Context Preservation**: Fixed semantic grouping to keep introduction lines connected to their lists
- **Clippy Warnings**: Addressed all linting issues for cleaner, more maintainable code

### Documentation
- Updated README.md with performance metrics and detailed usage examples
- Enhanced PRD.md with debug visualization documentation
- Added comprehensive inline code documentation

### Development
- **CI/CD**: Added GitHub Actions workflow for automated testing
- **Code Quality**: Implemented clippy linting with strict warning policies
- **Nix Integration**: Improved development environment setup with proper dependencies

### Performance
- Added profiling capabilities for performance monitoring

## [0.1.0] - 2025-07-07

### Added
- Initial release of rule72 commit message reflow tool
- Core text reflow functionality with configurable width (default: 72 characters)
- Support for preserving Git commit message structure (headline, body, footers)
- Unicode-aware text wrapping and width calculation
- List item detection and proper formatting
- Command-line interface with customizable options:
  - `--width`: Set body wrap width
  - `--headline-width`: Advisory headline width
  - `--no-ansi`: Strip ANSI color codes
- Nix package definition for easy installation
- Basic test suite and project structure
- MIT/Apache-2.0 dual licensing

### Dependencies
- clap 4.4+ for command-line argument parsing
- regex 1.0+ for pattern matching
- unicode-segmentation 1.10+ for proper text boundary detection
- unicode-width 0.1+ for accurate character width calculation
- anyhow 1.0+ for error handling

---

## Release Notes

### v0.2.0 Highlights

This release significantly improves the accuracy and usefulness of commit message reflow:

- **🔧 Major Bug Fix**: Resolved content truncation issue that was incorrectly treating regular content as Git footers
- **📊 Debug Visualization**: New SVG output helps understand how the tool parses and classifies your commit messages
- **🎯 Better Semantic Grouping**: Introduction lines are now properly connected to their following lists
- **⚡ Enhanced Performance**: Added profiling tools while maintaining fast processing speeds
- **🏗️ Improved Developer Experience**: Better Nix integration, comprehensive testing, and cleaner code

### Breaking Changes
None - this release maintains full backward compatibility with v0.1.0.

### Migration Notes
No migration required. All existing command-line options and behaviors are preserved.

### Contributors
- Mark Ruvald Pedersen (@wabsie)
- Claude Code Assistant (@anthropic)
