# Async Initialization

Lambda managed runtimes offer up to 10 seconds for function initialization with burst CPU. If your function can't complete initialization within that window, Lambda restarts it and bills for the init time.

## How It Works

When `AWS_LWA_ASYNC_INIT` is enabled:

1. The adapter performs readiness checks for up to 9.8 seconds
2. If the app isn't ready by then, the adapter signals Lambda that init is complete
3. Readiness checking continues during the first handler invocation
4. This avoids the restart penalty while using the free init CPU burst

## Enabling

```
AWS_LWA_ASYNC_INIT=true
```

## When to Use

Enable this when your application has a long startup time (e.g. loading large ML models, warming caches, establishing connection pools) that might exceed the 10-second init window.
