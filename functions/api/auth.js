export async function onRequestGet(context) {
  const { request, env } = context;

  const url = new URL(request.url);
  const customerId = url.searchParams.get("customer");

  if (!customerId) {
    return Response.json(
      { error: "missing customer" },
      { status: 400 }
    );
  }

  const status = await env.KEETANET_AUTH.get(customerId);

  return Response.json({
    customer: customerId,
    status: status || "unknown"
  });
}
