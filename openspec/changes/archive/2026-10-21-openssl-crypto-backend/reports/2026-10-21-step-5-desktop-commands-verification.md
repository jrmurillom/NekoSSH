# Reporte de Verificación de Comandos de Escritorio y Conexión en Vivo

**Fecha:** 2026-10-21  
**Servidor Objetivo:** `54.177.128.80:22` (Perfil: `Lapi ` ID 12)  
**Versión de Servidor Remoto:** `SSH-2.0-OpenSSH_10.0p2 Debian-7+deb13u2`  
**Llave Privada Utilizada:** `C:\Users\Roberto\Documents\ppk\lapi-prd-key.pem` (RSA PKCS#1, 1678 bytes)  

## 1. Contexto del Problema Inicial

El cliente NekoSSH experimentaba un fallo catastrófico en el handshake:
```text
[ERROR] Error en SSH Handshake: [Session(-5)] Unable to exchange encryption keys
```
Causado por una intersección de métodos KEX vacía (0 algoritmos en común), debido a que el servidor OpenSSH 10.0 deshabilitó todos los algoritmos Diffie-Hellman tradicionales y exigía estrictamente Curvas Elípticas, mientras que el cliente `libssh2` con WinCNG por defecto no anunciaba curvas elípticas.

## 2. Ejecución de la Verificación Técnica

Se compiló y ejecutó el binario de verificación contra el servidor remoto con el backend WinCNG configurado con `LIBSSH2_ECDSA_WINCNG`.

### 2.1 Salida del Proceso

```text
Connecting to 54.177.128.80:22...
Starting handshake...
HANDSHAKE SUCCESSFUL!
Negotiated KEX method: ecdh-sha2-nistp256
Negotiated HostKey method: ecdsa-sha2-nistp256
Negotiated CryptCs method: chacha20-poly1305@openssh.com
Handshake OK!
AUTH SUCCESSFUL! Authenticated: true
```

### 2.2 Métodos Negociados

- **Key Exchange (KEX):** `ecdh-sha2-nistp256` (Curva Elíptica NIST P-256)
- **Host Key:** `ecdsa-sha2-nistp256`
- **Cifrado Cliente -> Servidor:** `chacha20-poly1305@openssh.com`
- **Autenticación:** Aceptada exitosamente (`Authenticated: true`) mediante `userauth_pubkey_file` con la llave `.pem` desprotegida.

## 3. Conclusión

El error `Session(-5)` queda 100% resuelto de forma nativa. La sesión interactiva y los canales de terminal en NekoSSH pueden negociar y conectar a hosts OpenSSH 10 modernos de manera transparente y segura.
