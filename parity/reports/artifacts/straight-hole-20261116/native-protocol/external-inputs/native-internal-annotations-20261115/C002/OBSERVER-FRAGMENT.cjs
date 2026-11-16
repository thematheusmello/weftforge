async function observe(controller, app, label, mark, output) {
  var capture = await controller.capture(app, label);
  mark(capture.sourceRecord);
  output.write({sourceRecord: capture.sourceRecord, status: capture.status, state: capture.state});
  await output.emitImage({bytes: capture.images[0].bytes, mimeType: capture.images[0].mimeType});
  return capture;
}
module.exports = {observe};
