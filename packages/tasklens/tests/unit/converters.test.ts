import {readdirSync, readFileSync} from 'node:fs';
import {join} from 'node:path';
import {load} from 'js-yaml';
import {describe, expect, it} from 'vitest';
import type {TunnelAlgorithmFeatureSchema} from '../../src/generated/feature';
import type {TunnelAlgorithmTestCaseSchema} from '../../src/generated/test-case';
import {convertFeatureToLegacy, convertLegacyToFeature} from './converters';

const FIXTURES_PATH = join(process.cwd(), 'specs', 'compliance', 'fixtures');

describe('Converter Round-Trip Tests', () => {
  const allFiles = readdirSync(FIXTURES_PATH).filter(f => f.endsWith('.yaml'));

  describe('Legacy Fixtures Round-Trip', () => {
    const legacyFiles = allFiles.filter(f => !f.endsWith('.feature.yaml'));

    it('should have legacy fixtures to test', () => {
      expect(legacyFiles.length).toBeGreaterThan(0);
    });

    for (const file of legacyFiles) {
      it(`should round-trip legacy fixture: ${file}`, () => {
        const content = readFileSync(join(FIXTURES_PATH, file), 'utf8');
        const originalLegacy = load(content) as TunnelAlgorithmTestCaseSchema;

        // 1. Legacy -> Feature
        const feature = convertLegacyToFeature(originalLegacy);

        // 2. Feature -> Legacy[]
        const resultingLegacies = convertFeatureToLegacy(feature);

        // Expect exactly one resulting legacy case
        expect(resultingLegacies).toHaveLength(1);
        const roundTrippedLegacy = resultingLegacies[0];

        // 3. Compare
        // We might need to scrub undefineds or ensuring loose equality if properties are optional
        expect(roundTrippedLegacy).toEqual(originalLegacy);
      });
    }
  });

  describe('Feature Fixtures -> Legacy -> Round-Trip', () => {
    const featureFiles = allFiles.filter(f => f.endsWith('.feature.yaml'));

    it('should have feature fixtures to test', () => {
      expect(featureFiles.length).toBeGreaterThan(0);
    });

    for (const file of featureFiles) {
      it(`should round-trip derived legacy cases from: ${file}`, () => {
        const content = readFileSync(join(FIXTURES_PATH, file), 'utf8');
        const originalFeature = load(content) as TunnelAlgorithmFeatureSchema;

        // 1. Feature -> Legacy[]
        const derivedLegacies = convertFeatureToLegacy(originalFeature);

        // Ensure we actually got some test cases
        expect(derivedLegacies.length).toBeGreaterThan(0);

        for (const derivedLegacy of derivedLegacies) {
          // 2. Legacy -> Feature
          const tempFeature = convertLegacyToFeature(derivedLegacy);

          // 3. Feature -> Legacy[]
          const roundTrippedLegacies = convertFeatureToLegacy(tempFeature);

          // Expect exactly one legacy case back
          expect(roundTrippedLegacies).toHaveLength(1);
          const roundTrippedLegacy = roundTrippedLegacies[0];

          // 4. Compare
          expect(roundTrippedLegacy).toEqual(derivedLegacy);
        }
      });
    }
  });
});
