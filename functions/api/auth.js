export async function onRequestGet(context) {
  const { request, env } = context;

  const url = new URL(request.url);
  const customerId = url.searchParams.get("customer");

  if (!customerId) {
    return Response.json(
      {
        error: "missing customer"
      },
      {
        status: 400
      }
    );
  }

  const data = await env.KEETANET_AUTH.get(customerId);

  if (!data) {
    return Response.json({
      customer: customerId,
      status: "unknown"
    });
  }

  try {
    return Response.json(JSON.parse(data));
  } catch {
    return Response.json({
      customer: customerId,
      status: data
    });
  }
}
