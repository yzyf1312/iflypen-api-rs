# Iflyrec CLI Tool

A command-line interface for interacting with the iFlyRec API, designed to submit audio transcription tasks.

---

## 📦 Features
- Audio file upload and transcription task creation
- Custom vocabulary optimization (supports both Chinese/English commas)
- Multilingual support (Chinese/English)
- SMS notifications
- Automatic extraction of most recent session_id
- Download transcription results

---

## 📝 Usage Guide

### 1. Prerequisites
#### Obtain Cookies Database
- **Windows**: Copy `%APPDATA%\讯飞听见\Cookies` file to working directory
- Other platforms: Path unknown

#### Prepare Audio File
Rename your audio file to `data.mp3` (format support determined by iFlyRec server)

---

### 2. Basic Commands
```bash
# Minimal usage
./iflyrec-cli -f data.mp3

# Full parameter example
./iflyrec-cli -f data.mp3 -n "Meeting" -w "Rust,WebRTC,AI" -l cn -s -d Cookies

# Download result by order ID
./iflyrec-cli -o 1234567890
```

---

## 📌 Parameter Reference

| Param | Full Name    | Description                  | Default        |
| ----- | ------------ | ---------------------------- | -------------- |
| `-f`  | `--file`     | Path to audio file           | None           |
| `-n`  | `--name`     | Transcription task name      | Auto-generated |
| `-w`  | `--hotwords` | Comma-separated vocabulary   | Empty          |
| `-l`  | `--lang`     | Language code                | `cn`           |
| `-s`  | `--sms`      | Enable SMS notifications     | Disabled       |
| `-d`  | `--db`       | Cookies database path        | `Cookies`      |
| `-o`  | `--order-id` | en: Order ID (download mode) | None           |

---

## 📂 Base File Structure
```
./
 ├── iflyrec-cli       # Executable binary
 ├── Cookies             # iFlyRec session database
 └── data.mp3            # Audio to transcribe
```

---

## ⚠️ Important Notes
1. Must maintain active iFlyRec account login
2. Maximum hotword limit unspecified
4. Re-login required in iFlyRec client to refresh database when session_id expires

---

## 📋 Troubleshooting

| Error Message               | Solution                          |
| --------------------------- | --------------------------------- |
| `No valid session_id found` | Verify Cookies path and validity  |
| `Database access error`     | Check file permissions/locking    |
| `Transcription failed`      | Verify network and account status |

---

## 🧪 Example Scenarios

### Example 1: Basic Transcription
```bash
./iflyrec-cli -f data.mp3
```

### Example 2: With Vocabulary and SMS
```bash
./iflyrec-cli -f data.mp3 -w "AI,Machine Learning" -s
```

### Example 3: English Meeting Notes
```bash
./iflyrec-cli -f meeting.wav -n "EngMeeting" -l en
```

---

For issues or contributions, please open an issue or submit a pull request.