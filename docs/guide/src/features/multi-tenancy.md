# Multi-Tenancy

Lambda Web Adapter supports multi-tenancy by automatically propagating the tenant ID from the Lambda runtime context to your web application.

## How It Works

When the Lambda runtime includes a `tenant_id` in the invocation context, the adapter forwards it as an `X-Amz-Tenant-Id` HTTP header. If no tenant ID is present, the header is omitted.

## Reading the Tenant ID

```python
# FastAPI
@app.get("/")
def handler(request: Request):
    tenant_id = request.headers.get("x-amz-tenant-id")
```

```javascript
// Express.js
app.get('/', (req, res) => {
    const tenantId = req.headers['x-amz-tenant-id'];
});
```

No additional configuration is required.
