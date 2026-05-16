import { expect, test } from 'bun:test';
import { formatTime, parseSrtTime, formatSrtTime, parseAssTime, formatAssTime, formatFileSize, formatDuration, truncateText, capitalizeFirst, slugify, getFileExtension, stripHtmlTags, stripAssTags, formatBytes } from './format';

test('formatTime formats seconds correctly', () => {
  expect(formatTime(0)).toBe('00:00');
  expect(formatTime(59)).toBe('00:59');
  expect(formatTime(60)).toBe('01:00');
  expect(formatTime(3661)).toBe('01:01:01');
  expect(formatTime(3600)).toBe('01:00:00');
  expect(formatTime(7325)).toBe('02:02:05');
});

test('parseSrtTime parses SRT timestamp to seconds', () => {
  expect(parseSrtTime('00:00:00,000')).toBe(0);
  expect(parseSrtTime('00:00:01,000')).toBe(1);
  expect(parseSrtTime('00:01:00,000')).toBe(60);
  expect(parseSrtTime('01:00:00,000')).toBe(3600);
  expect(parseSrtTime('01:23:45,678')).toBe(5025.678);
  expect(parseSrtTime('invalid')).toBe(0);
});

test('formatSrtTime formats seconds to SRT timestamp', () => {
  expect(formatSrtTime(0)).toBe('00:00:00,000');
  expect(formatSrtTime(1)).toBe('00:00:01,000');
  expect(formatSrtTime(60)).toBe('00:01:00,000');
  expect(formatSrtTime(3600)).toBe('01:00:00,000');
});

test('parseAssTime parses ASS timestamp to seconds', () => {
  expect(parseAssTime('00:00:00.00')).toBe(0);
  expect(parseAssTime('00:00:01.00')).toBe(1);
  expect(parseAssTime('00:01:00.00')).toBe(60);
  expect(parseAssTime('01:00:00.00')).toBe(3600);
  expect(parseAssTime('01:23:45.67')).toBe(5025.67);
  expect(parseAssTime('invalid')).toBe(0);
});

test('formatAssTime formats seconds to ASS timestamp', () => {
  expect(formatAssTime(0)).toBe('00:00:00.00');
  expect(formatAssTime(1)).toBe('00:00:01.00');
  expect(formatAssTime(60)).toBe('00:01:00.00');
  expect(formatAssTime(3600)).toBe('01:00:00.00');
  expect(formatAssTime(5025.67)).toBe('01:23:45.67');
});

test('formatFileSize formats bytes correctly', () => {
  expect(formatFileSize(0)).toBe('0 B');
  expect(formatFileSize(512)).toBe('512 B');
  expect(formatFileSize(1024)).toBe('1.0 KB');
  expect(formatFileSize(1536)).toBe('1.5 KB');
  expect(formatFileSize(1048576)).toBe('1.0 MB');
  expect(formatFileSize(1572864)).toBe('1.5 MB');
});

test('formatDuration formats seconds to human readable', () => {
  expect(formatDuration(0)).toBe('0s');
  expect(formatDuration(59)).toBe('59s');
  expect(formatDuration(60)).toBe('1m 0s');
  expect(formatDuration(3600)).toBe('1h 0s');
  expect(formatDuration(3661)).toBe('1h 1m 1s');
  expect(formatDuration(7325)).toBe('2h 2m 5s');
});

test('truncateText truncates long text', () => {
  expect(truncateText('Short', 10)).toBe('Short');
  expect(truncateText('This is a long text that should be truncated', 10)).toBe('This is...');
  expect(truncateText('ABCDEFGHIJ', 5)).toBe('AB...');
});

test('capitalizeFirst capitalizes first letter', () => {
  expect(capitalizeFirst('hello')).toBe('Hello');
  expect(capitalizeFirst('WORLD')).toBe('World');
  expect(capitalizeFirst('a')).toBe('A');
  expect(capitalizeFirst('')).toBe('');
});

test('slugify converts text to slug', () => {
  expect(slugify('Hello World')).toBe('hello-world');
  expect(slugify('This is a TEST')).toBe('this-is-a-test');
  expect(slugify('special!@#$chars')).toBe('specialchars');
  expect(slugify('multiple   spaces')).toBe('multiple-spaces');
  expect(slugify('---leading-dashes---')).toBe('leading-dashes');
});

test('getFileExtension extracts extension', () => {
  expect(getFileExtension('file.srt')).toBe('srt');
  expect(getFileExtension('file.ASS')).toBe('ass');
  expect(getFileExtension('video.mkv')).toBe('mkv');
  expect(getFileExtension('multiple.dots.file.srt')).toBe('srt');
});

test('stripHtmlTags removes HTML tags', () => {
  expect(stripHtmlTags('<b>Hello</b>')).toBe('Hello');
  expect(stripHtmlTags('<p>Paragraph</p>')).toBe('Paragraph');
  expect(stripHtmlTags('<div class="test">Content</div>')).toBe('Content');
  expect(stripHtmlTags('No tags here')).toBe('No tags here');
  expect(stripHtmlTags('<br/>')).toBe('');
});

test('stripAssTags removes ASS override tags', () => {
  expect(stripAssTags('Hello \\pos(100,200) World')).toBe('Hello World');
  expect(stripAssTags('Text \\blur1 \\frz10.5')).toBe('Text');
  expect(stripAssTags('Style \\fscx61 \\fscy50')).toBe('Style');
  expect(stripAssTags('\\N line break')).toContain('line break');
  expect(stripAssTags('soft \\n break')).toBe('soft break');
  expect(stripAssTags('')).toBe('');
});

test('formatBytes formats bytes correctly', () => {
  expect(formatBytes(0)).toBe('0 Bytes');
  expect(formatBytes(1024)).toBe('1 KB');
  expect(formatBytes(1536, 1)).toBe('1.5 KB');
  expect(formatBytes(1048576)).toBe('1 MB');
  expect(formatBytes(1572864, 2)).toBe('1.5 MB');
});