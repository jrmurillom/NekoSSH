import { describe, expect, it } from "vitest";
import {
  clampPercent,
  formatBytes,
  formatSpeed,
  formatTransferHeader,
  formatTransferMetrics,
  isTransferCancelledError,
  TRANSFER_CANCELLED_ERROR,
  type TransferProgressPayload,
} from "./transfer-progress-helper";

describe("transfer-progress-helper", () => {
  it("formatea bytes en B, KB, MB y GB de forma legible", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(-10)).toBe("0 B");
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(1536)).toBe("1.5 KB");
    expect(formatBytes(10 * 1024 * 1024)).toBe("10.0 MB");
    expect(formatBytes(2.5 * 1024 * 1024 * 1024)).toBe("2.50 GB");
  });

  it("formatea velocidades en B/s, KB/s y MB/s", () => {
    expect(formatSpeed(0)).toBe("0 B/s");
    expect(formatSpeed(2048)).toBe("2.0 KB/s");
    expect(formatSpeed(4.2 * 1024 * 1024)).toBe("4.2 MB/s");
  });

  it("normaliza porcentajes en el rango 0..100", () => {
    expect(clampPercent(-5)).toBe(0);
    expect(clampPercent(0)).toBe(0);
    expect(clampPercent(42.7)).toBe(43);
    expect(clampPercent(150)).toBe(100);
    expect(clampPercent(Number.NaN)).toBe(0);
  });

  it("construye encabezado diferenciado para subida individual, por lote y copia SCP", () => {
    const uploadSingle: TransferProgressPayload = {
      operation: "upload",
      file_name: "archive.zip",
      bytes_transferred: 5242880,
      total_bytes: 10485760,
      percent: 50,
      speed_bps: 1048576,
    };

    expect(formatTransferHeader(uploadSingle, 1, 1)).toEqual({
      titleText: "Subiendo archive.zip",
      percentText: "50%",
    });

    expect(formatTransferHeader(uploadSingle, 2, 5)).toEqual({
      titleText: "Subiendo archive.zip (2/5)",
      percentText: "50%",
    });

    const scpPayload: TransferProgressPayload = {
      operation: "scp",
      file_name: "dump.sql",
      bytes_transferred: 10485760,
      total_bytes: 10485760,
      percent: 100,
      speed_bps: 2097152,
    };

    expect(formatTransferHeader(scpPayload)).toEqual({
      titleText: "Copiando SCP dump.sql",
      percentText: "100%",
    });

    const downloadPayload: TransferProgressPayload = {
      operation: "download",
      file_name: "prod-backup.sql.gz",
      bytes_transferred: 6.4 * 1024 * 1024 * 1024,
      total_bytes: 10 * 1024 * 1024 * 1024,
      percent: 64,
      speed_bps: 42 * 1024 * 1024,
    };

    expect(formatTransferHeader(downloadPayload)).toEqual({
      titleText: "Descargando prod-backup.sql.gz",
      percentText: "64%",
    });
  });

  it("construye línea de métricas con bytes transferidos y velocidad", () => {
    const active: TransferProgressPayload = {
      operation: "upload",
      file_name: "video.mp4",
      bytes_transferred: 5 * 1024 * 1024,
      total_bytes: 20 * 1024 * 1024,
      percent: 25,
      speed_bps: 2.5 * 1024 * 1024,
    };
    expect(formatTransferMetrics(active)).toBe("5.0 MB / 20.0 MB • 2.5 MB/s");

    const zeroSpeed: TransferProgressPayload = {
      ...active,
      bytes_transferred: 0,
      percent: 0,
      speed_bps: 0,
    };
    expect(formatTransferMetrics(zeroSpeed)).toBe("0 B / 20.0 MB");

    const preFlight: TransferProgressPayload = {
      operation: "upload",
      file_name: "video.mp4",
      bytes_transferred: 0,
      total_bytes: 0,
      percent: 0,
      speed_bps: 0,
    };
    expect(formatTransferMetrics(preFlight)).toBe("Preparando transferencia…");

    const emptyFileCompleted: TransferProgressPayload = {
      operation: "upload",
      file_name: "empty.txt",
      bytes_transferred: 0,
      total_bytes: 0,
      percent: 100,
      speed_bps: 0,
    };
    expect(formatTransferMetrics(emptyFileCompleted)).toBe("0 B / 0 B");
  });

  it("detecta errores de cancelación cooperativa TRANSFER_CANCELLED sin confundirlos con fallos reales", () => {
    expect(isTransferCancelledError(TRANSFER_CANCELLED_ERROR)).toBe(true);
    expect(isTransferCancelledError(new Error(`Error: ${TRANSFER_CANCELLED_ERROR}`))).toBe(true);
    expect(isTransferCancelledError("Permission denied")).toBe(false);
    expect(isTransferCancelledError(null)).toBe(false);
    expect(isTransferCancelledError(undefined)).toBe(false);
  });
});
