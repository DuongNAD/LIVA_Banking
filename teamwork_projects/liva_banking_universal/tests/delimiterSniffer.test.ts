import { describe, it, expect } from 'vitest';
import { sniffDelimiter, stripBom, countDelimitersOutsideQuotes, splitCsvLine } from '../src/engine/ingestion/delimiterSniffer';

describe('DelimiterSniffer', () => {
  it('should strip UTF-8 BOM correctly', () => {
    const withBom = '\uFEFFcol1;col2;col3';
    const res = stripBom(withBom);
    expect(res.encoding).toBe('UTF-8-BOM');
    expect(res.cleaned).toBe('col1;col2;col3');
  });

  it('should count delimiters strictly outside of quotes', () => {
    const line = '"Company, Ltd";"Field; with semicolon";12345';
    const counts = countDelimitersOutsideQuotes(line);
    expect(counts[';']).toBe(2);
    expect(counts[',']).toBe(0);
  });

  it('should detect semicolon delimiter in standard Vietnamese CSV', () => {
    const sample = `Mã GD;Ngày giao dịch;Số tiền;Nội dung\nTX01;01/08/2026;1000000;Thanh toán tiền\nTX02;02/08/2026;2000000;Nạp tiền`;
    const res = sniffDelimiter(sample);
    expect(res.delimiter).toBe(';');
  });

  it('should detect tab delimiter in clipboard data', () => {
    const sample = `Mã GD\tNgày giao dịch\tSố tiền\tNội dung\nTX01\t01/08/2026\t1000000\tThanh toan`;
    const res = sniffDelimiter(sample);
    expect(res.delimiter).toBe('\t');
  });

  it('should split CSV line preserving escaped quotes', () => {
    const line = 'VCB26214001;"CONG TY ""AN PHAT"" TNHH";145000000';
    const cells = splitCsvLine(line, ';');
    expect(cells.length).toBe(3);
    expect(cells[0]).toBe('VCB26214001');
    expect(cells[1]).toBe('CONG TY "AN PHAT" TNHH');
    expect(cells[2]).toBe('145000000');
  });

  it('should preserve inner quotes at cell boundaries without redundant stripping', () => {
    const lineEnd = 'TX02;"Ghi chu: ""Chuyen khoan""";5000000';
    const cells = splitCsvLine(lineEnd, ';');
    expect(cells.length).toBe(3);
    expect(cells[1]).toBe('Ghi chu: "Chuyen khoan"');
  });
});
