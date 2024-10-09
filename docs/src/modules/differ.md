# Differ


### Code diffs

To diff between what is locally in your current directory (function) and remote lambda in a sandbox

```
tc diff -s <sandbox> -e <env>

```

![Diff](/images/diff.png)


### Changelog

```
cd services/extraction/dealer
tc diff --changelog


[QA-1200] Fixed quotes for the suffix conditional statement (#5258)

0.26.0
[STP-3166] Add all Credit Union Names to lender name extractions (#5212)
[QA-1186] [QA] For bookout sheet incorrect key used for clean_trade_in_price (#5198)
[STP-3179] handle serialize-via-cp Lambda.ServiceException (#5178)
(#5172)

0.25.0
[STP-3064] Add support for new bookout_sheet extraction keys and integrate ML model for all existing fields as well. (#5139)
[STP-3165] FinbeUSA lender name (#5115)
Add extracted_data_uri in serializer output always (#5112)
```
