"use strict";

let failures = 0;

if (!"The future is cool!".startsWith("The ")) failures |= 1;
if (!"The future is cool!".startsWith("future", 4)) failures |= 2;
if ("The future is cool!".startsWith("future")) failures |= 4;
if (!"The future is cool!".endsWith("cool!")) failures |= 8;
if (!"The future is cool!".endsWith("future", 10)) failures |= 16;
if ("The future is cool!".endsWith("future")) failures |= 32;
if (!"abc".startsWith("", 99)) failures |= 64;
if (!"abc".endsWith("", -1)) failures |= 128;

let startsLength = Object.getOwnPropertyDescriptor(String.prototype.startsWith, "length");
if (String.prototype.startsWith.length !== 1) failures |= 256;
if (startsLength.value !== 1 || startsLength.writable || startsLength.enumerable || !startsLength.configurable) failures |= 512;

let endsLength = Object.getOwnPropertyDescriptor(String.prototype.endsWith, "length");
if (String.prototype.endsWith.length !== 1) failures |= 1024;
if (endsLength.value !== 1 || endsLength.writable || endsLength.enumerable || !endsLength.configurable) failures |= 2048;

failures === 0;
