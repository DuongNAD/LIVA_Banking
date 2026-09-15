/**
 * Quote-Aware Delimiter Sniffer & Text Line Tokenizer
 * Handles UTF-8 BOM, UTF-16 BOM, semicolon, comma, tab, pipe, and quoted CSV fields.
 */

export interface SnifferResult {
  delimiter: string;
  encoding: 'UTF-8' | 'UTF-8-BOM' | 'UTF-16';
  cleanedContent: string;
  lineCount: number;
}

const SUPPORTED_DELIMITERS = [';', '\t', ',', '|'];

/**
 * Strips BOM markers and cleans initial whitespace.
 */
export function stripBom(text: string): { cleaned: string; encoding: 'UTF-8' | 'UTF-8-BOM' | 'UTF-16' } {
  if (text.charCodeAt(0) === 0xfeff || text.startsWith('\xef\xbb\xbf')) {
    return {
      cleaned: text.replace(/^(\uFEFF|\xEF\xBB\xBF)/, ''),
      encoding: 'UTF-8-BOM',
    };
  }
  if (text.charCodeAt(0) === 0xfffe) {
    return {
      cleaned: text.slice(1),
      encoding: 'UTF-16',
    };
  }
  return {
    cleaned: text,
    encoding: 'UTF-8',
  };
}

/**
 * Counts occurrences of each delimiter strictly OUTSIDE of quoted strings.
 */
export function countDelimitersOutsideQuotes(line: string): Record<string, number> {
  const counts: Record<string, number> = {
    ';': 0,
    '\t': 0,
    ',': 0,
    '|': 0,
  };

  let inQuotes = false;
  let quoteChar = '';

  for (let i = 0; i < line.length; i++) {
    const char = line[i];

    if (!inQuotes && (char === '"' || char === "'")) {
      inQuotes = true;
      quoteChar = char;
    } else if (inQuotes && char === quoteChar) {
      // Check for escaped quote (e.g. "")
      if (i + 1 < line.length && line[i + 1] === quoteChar) {
        i++; // skip next quote
      } else {
        inQuotes = false;
      }
    } else if (!inQuotes && counts[char] !== undefined) {
      counts[char]++;
    }
  }

  return counts;
}

/**
 * Sniffs the primary delimiter by analyzing the first N non-empty lines.
 */
export function sniffDelimiter(rawText: string, maxSampleLines = 15): SnifferResult {
  const { cleaned, encoding } = stripBom(rawText);
  const lines = cleaned.split(/\r?\n/).filter((l) => l.trim().length > 0);

  if (lines.length === 0) {
    return {
      delimiter: ',',
      encoding,
      cleanedContent: cleaned,
      lineCount: 0,
    };
  }

  const sampleLines = lines.slice(0, Math.min(lines.length, maxSampleLines));
  const delimiterScores: Record<string, number> = {
    ';': 0,
    '\t': 0,
    ',': 0,
    '|': 0,
  };

  // Track consistency: how many lines have identical count > 0 for this delimiter
  const delimiterDistributions: Record<string, number[]> = {
    ';': [],
    '\t': [],
    ',': [],
    '|': [],
  };

  for (const line of sampleLines) {
    const lineCounts = countDelimitersOutsideQuotes(line);
    for (const d of SUPPORTED_DELIMITERS) {
      const cnt = lineCounts[d];
      delimiterScores[d] += cnt;
      if (cnt > 0) {
        delimiterDistributions[d].push(cnt);
      }
    }
  }

  // Evaluate candidate delimiters
  let bestDelimiter = ',';
  let bestScore = -1;

  for (const d of SUPPORTED_DELIMITERS) {
    const totalCount = delimiterScores[d];
    const occurrences = delimiterDistributions[d];
    if (occurrences.length === 0) continue;

    // Check consistency: percentage of sample lines containing this delimiter
    const coverage = occurrences.length / sampleLines.length;

    // Standard deviation / consistency of column counts across rows
    const firstCount = occurrences[0];
    const consistentRows = occurrences.filter((c) => c === firstCount).length;
    const consistency = consistentRows / occurrences.length;

    // Weight score: total count * coverage * (1 + consistency)
    const compositeScore = totalCount * coverage * (1 + consistency);

    if (compositeScore > bestScore) {
      bestScore = compositeScore;
      bestDelimiter = d;
    }
  }

  return {
    delimiter: bestDelimiter,
    encoding,
    cleanedContent: cleaned,
    lineCount: lines.length,
  };
}

/**
 * Splits a CSV/TSV line into individual cell tokens respecting quoted text.
 */
export function splitCsvLine(line: string, delimiter: string): string[] {
  const cells: string[] = [];
  let current = '';
  let inQuotes = false;
  let quoteChar = '';

  for (let i = 0; i < line.length; i++) {
    const char = line[i];

    if (!inQuotes && (char === '"' || char === "'")) {
      inQuotes = true;
      quoteChar = char;
    } else if (inQuotes && char === quoteChar) {
      if (i + 1 < line.length && line[i + 1] === quoteChar) {
        current += quoteChar;
        i++;
      } else {
        inQuotes = false;
      }
    } else if (!inQuotes && char === delimiter) {
      cells.push(current.trim());
      current = '';
    } else {
      current += char;
    }
  }

  cells.push(current.trim());
  return cells;
}
