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

  const data = await env.KEETANET_AUTH.get(customerId);

  if (!data) {
    return Response.json({
      customer: customerId,
      status: "unknown"
    });
  }

  try {
    const license = JSON.parse(data);

    if (
      license.expires_at &&
      new Date(license.expires_at) < new Date()
    ) {
      license.status = "expired";
    }

    return Response.json(license);

  } catch {
    return Response.json({
      customer: customerId,
      status: data
    });
  }
}
