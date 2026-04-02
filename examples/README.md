# Examples

## command_bot.py

Contoh automation command sederhana dengan alur:

- menerima pesan masuk (`EvMessage`)
- parsing command dari teks pesan
- reply dengan `quoted` message
- ambil profile picture pengirim (`contact.get_profile_picture`) lalu kirim kembali sebagai foto
- ambil pushname dari metadata pesan (`message_info.push_name`)
- ambil bio/about pengirim (`contact.get_user_info`)

### Command yang didukung

- `ping` -> client membalas `pong`
- `pp` -> client download profile picture pengirim dan kirim balik ke chat
- `pushname` -> client membalas pushname pengirim
- `bio` -> client membalas bio/about pengirim
- `help` / `menu` -> menampilkan daftar command

### Menjalankan

```bash
python examples/command_bot.py
```

Opsional environment variable:

- `TRYX_DB_PATH` (default: `whatsapp.db`)
