# Releaser

<!-- toc -->

## A simple Release workflow

The following is a simple release workflow:

1. A PR merge creates a `Patch Release` (example 0.1.1, 0.1.2 so forth)
2. An explicit release using the following command creates a `Minor Release` (example 0.2.0)

   ```sh
   tc release --service extraction
   ```
   This triggers CI to create an annotated tag. An [annotated tag](https://git-scm.com/book/en/v2/Git-Basics-Tagging) is different from a lightweight or commit tag.

3. This stable `Minor Release` can be deployed to other environments and sandboxes

   ```sh
   tc deploy --service extraction-dealer --version 0.2.0 --sandbox stable --env qa
   ```

```admonish info
Minor releases are annotated git tags. They are not generated via a commit, but are annotations on the last known good patch release.
```

## Git tagging

```
Options:
  -n, --next <NEXT>
  -s, --service <SERVICE>
      --dry-run
      --push
  -S, --suffix <SUFFIX>
```

To create next patch version:

```sh
cd services/extraction/dealer
tc tag --next patch --service extraction-dealer [--dry-run]

Tag { prefix: "extraction", parent: "0.1.8", create: true, version: "0.1.9" }
```

To create a minor release from a latest rc tag

```sh
tc release --next minor --service extraction --dry-run
Tag { prefix: "extraction", parent: "0.1.8", create: true, version: "0.2.0" }
```


```admonish info
A topology or service is typically tagged in github. Nano functions don't have a git tag, instead are part of the manifest with revisions or git-less semvers
```

## Changelog

### Annotation-based changelog

tc leverages annotation tags to get accurate changes and diffs between two stable releases.

```
tc changelog --service extraction-dealer
[Infra] Feature/extraction fallback output (#2985)
[STP-2732] [Mountain CU POC Env] Bring lender name extractions back to benchmark by adding them to the extraction context (#2977)
[STP-2728] [CUDirect] Clean Retail Price Extraction Precision Issues (#2942)

0.7.0
[QA] RISC incorrect dealer address extracted and verification failing (#2960)

0.6.0
[QA-895] Fixed applicant 2 address bug (#2953)

0.5.0
[STP-2733][CUDirect POC Env] Lender name being extracted incorrectly from Oregon Title Apps (#2943)
[STP] Undo provider address and phone number extraction (#2938)
[STP-2709] Address extraction improvements for Y12 credit app format (#2944)
[LIV-1280] Assigned without recourse extraction is stricter (#2929)
[LIV-1310][LIV-1316] [LIV-1312] LIVOPS fixes (#2930)
support AP for mastery (#2876)
[QA-873] Dealer address is not being used in the verification (#2845)
```
`-v` shows each semver patch


To find the changelog between two versions (including annotated):

```sh
tc changelog --between 0.2.2..0.3.0

tc changelog --between 0.2.2-qa..0.2.10-qa
```

To find the revisions of all components in the service:

```sh
tc changelog -t --service extraction-dealer
document-extraction
 - revision: fdc1d48
 - lang: ruby2.7
 - layers:
 document-extraction-0
 document-extraction-1
 native
fallback
 - revision: fdc1d48
 - lang: python3.10
 - layers:
 python-lambda-base
field-grouper
 - revision: fdc1d48
 - lang: python3.10
field-mapper
```
