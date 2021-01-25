const express = require("express")
const net = require("net")

const server_http = express()
const server_tcp = net.createServer()

let http_c = 0
let tcp_c = 0

server_http.all("/ping", (req, res) => {
    http_c += 1
    console.log(`HTTP: ${http_c}`)
    res.send("PONG")
})


server_tcp.on("connection", (sock) => {
    try {
        sock.on("data", (data) => {
            tcp_c += 1
            console.log(`TCP: ${tcp_c} (${data.toString("utf8")})`)
        })
    } catch (err) {
        // Because the program doesn't gracefully terminate the TCP connection
        // Means we gotta' catch it :D
    }
})

server_http.listen(80)
server_tcp.listen(333)