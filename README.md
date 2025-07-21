# IFlyPen API Rust SDK

Own an iFlytek Smart Recorder and enjoy free services? Want to quickly convert streaming audio to text? Hoping to quickly summarize the content of your online meetings? With this project, you can bypass hardware limitations to quickly convert local audio files to text, automatically generate meeting minutes, and even call AI models for content summarization and translation—all without relying on iFlytek Smart Recorder hardware to enjoy lifetime free services.

## Project Description

This SDK is implemented based on the reverse engineering of the iFlyrec client protocol and is used to access the IFlyPen API. Key features:
- ✅ Submit local audio files for transcription (currently the only feature)
- ⚠️ Task status and results must be viewed through the official client
- 🚫 Result query interface has not been implemented yet (see the development roadmap for details)

### What is the IFlyPen API?

The IFlyPen API is a special version of the iFlyrec client API. This repository is its Rust implementation. This API version provides users who have purchased an iFlytek Smart Recorder with a variety of lifetime free services, such as speech-to-text, AI summarization, and full-text translation.

This project aims to remove iFlytek's restriction on this free API (which requires recordings to be made with an iFlytek Smart Recorder to be eligible for the benefits), providing convenienceenience for users.

## Quick Start

### Dependencies

```toml
cargo add --git https://github.com/iflypen/iflypen-api-rs
```

### Example

For details, see `src/bin/iflypen-cli.rs`. You can try it out by running `cargo run`.

## Development Roadmap

| Feature Module                       | Development Status | Target Version |
| ------------------------------------ | ------------------ | -------------- |
| Speech Transcription Task Submission | ✅ Implemented      | v1.0           |
| Task Result Query                    | 🔧 In Development   | v1.1           |
| Account History Access               | 🔧 In Development   | v1.2           |
| Multi-threaded Upload Support        | 🚧 Planned          | v1.3           |
| AI Summary Generation                | 🚧 Planned          | v1.4           |
| Real-time Translation Engine         | 🚧 Planned          | v1.5           |
| Batch Task Processing                | 🚧 Planned          | v1.6           |

## Warning

This project violates clauses **V-7-(2)-8)** and **V-7-(3)** of the [iFlyrec User Agreement](https://static.iflyrec.com/v1/iflyrectjpt/publicread01/privacyPolicy/tjzs/userPrivacyPolicy.html). Users may suffer losses for using this project to access iFlytek services. For specific penalties, please see **Section XI: Handling of Breach of Contract** of the iFlyrec User Agreement.

\*\*This project is for protocol analysis and technical research only. Please adhere to the official iFlytek terms of service. Any loss oramage resulting from the use of this project is the sole responsibility of the user.**

## Contribution Guide

This project accepts contributions in the following forms:

- Reverse engineering/packet capture samples and analysis results
- Documentation improvements and additional examples
- Bug fixes and logging optimizations

If you would like to contribute to this project or have any questions, feel free to open an issue or pull request.

## License

## This project is licensed under the MIT License, but all usage:
- Is not endorsed or supported by iFlytek.
- Must be undertaken at your own legal risk.
- Comes with no service guarantees.