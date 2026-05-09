# Tutorial 10 Asynchronous Programming (Bagian 2)

### Experiment 2.1: Original code of broadcast chat. 
Cara menjalankan: buka 4 terminal. Terminal 1 jalankan server dengan `cargo run --bin server`. Terminal 2, 3, 4 jalankan client dengan `cargo run --bin client`. Ketika kita mengetik pesan di salah satu client dan menekan Enter, server akan menerima pesan tersebut dan mem-broadcast ke semua client yang terhubung melalui channel broadcast Tokio.

Screenshots:
server
![server](server.png)

clients
![client1](client1.png)

![client2](client2.png)

![client3](client3.png)


### Experiment 2.2: Modifying the websocket port 
Port diubah di dua file: `server.rs` (di `TcpListener::bind`) dan `client.rs` (di `ClientBuilder::from_uri`). Keduanya menggunakan protokol WebSocket (`ws://`). Server mendefinisikan port yang di-listen, dan client harus mengarah ke port yang sama. Protokol WebSocket (`ws://`) didefinisikan di URI string pada client dan secara implisit di `ServerBuilder` pada server.

Screenshots:
server
![server-2](server-2.png)

clients
![client-2-1](client-2-1.png)

![client-2-2](client-2-2.png)

![client-2-3](client-2-3.png)


### Experiment 2.3: Small changes, add IP and Port
Perubahan dilakukan di `server.rs` pada fungsi `handle_connectio`n`. Sebelumnya pesan di-broadcast apa adanya. Sekarang pesan dibungkus dengan format!("{addr}: {text}") sehingga menyertakan IP dan port pengirim. Ini membantu client mengetahui siapa yang mengirim pesan, karena setiap koneksi TCP memiliki port unik yang mengidentifikasi masing-masing client.

Screenshots:
server
![server-3](server-3.png)

clients
![client-3-1](client-3-1.png)

![client-3-1](client-3-2.png)

![client-3-3](client-3-3.png)