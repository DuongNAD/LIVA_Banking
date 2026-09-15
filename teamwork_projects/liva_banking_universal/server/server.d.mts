import type { Server } from 'node:http';

export const server: Server;
export const USERS: any[];
export const DB: any;
export function signJwt(payload: any, expiresInSeconds?: number): string;
export function verifyJwt(token: string): any;
export function broadcastEvent(event: string, data: any): void;
export const sseClients: Set<any>;
export const PORT: number | string;
export function base64UrlEncode(str: string): string;
export function base64UrlDecode(str: string): string;
export default server;
