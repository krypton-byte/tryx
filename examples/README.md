# Examples

## command_bot.py

Contoh bot command sederhana dengan alur:

- menerima pesan masuk (`EvMessage`)
- parsing command dari teks pesan
- reply dengan `quoted` message
- ambil profile picture pengirim (`contact.get_profile_picture`) lalu kirim kembali sebagai foto
- ambil pushname dari metadata pesan (`message_info.push_name`)
- ambil bio/about pengirim (`contact.get_user_info`)

### Command yang didukung

- `ping` -> bot membalas `pong`
- `pp` -> bot download profile picture pengirim dan kirim balik ke chat
- `pushname` -> bot membalas pushname pengirim
- `bio` -> bot membalas bio/about pengirim
- `help` / `menu` -> menampilkan daftar command

### Menjalankan

```bash
python examples/command_bot.py
```

Opsional environment variable:

- `TRYX_DB_PATH` (default: `whatsapp.db`)
