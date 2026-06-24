export async function onRequestGet(context) {
  const { env } = context;

  return Response.json({
    hasSecret: !!env.STRIPE_WEBHOOK_SECRET,
    secretLength: env.STRIPE_WEBHOOK_SECRET?.length || 0,
    hasStripeKey: !!env.STRIPE_SECRET_KEY,
    hasKV: !!env.KEETANET_AUTH
  });
}
