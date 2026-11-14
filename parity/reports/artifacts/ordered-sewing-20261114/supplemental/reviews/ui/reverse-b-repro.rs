// Insert inside existing tests::ordered_sewing; reuses that module's fixture and harness helpers.
        fn pointer_click_label(ctx: &egui::Context, app: &mut Studio, output: &egui::FullOutput, label: &str) -> egui::FullOutput {
            let id = *buttons(output, label).last().unwrap();
            let nodes = &output.platform_output.accesskit_update.as_ref().unwrap().nodes;
            let bounds = nodes.iter().find(|(node_id, _)| *node_id == id).unwrap().1.bounds().unwrap();
            let pos = Pos2::new(((bounds.x0+bounds.x1)*0.5) as f32, ((bounds.y0+bounds.y1)*0.5) as f32);
            frame(ctx, app, vec![egui::Event::PointerMoved(pos), egui::Event::PointerButton { pos, button:egui::PointerButton::Primary, pressed:true, modifiers:egui::Modifiers::NONE }]);
            frame(ctx, app, vec![egui::Event::PointerMoved(pos), egui::Event::PointerButton { pos, button:egui::PointerButton::Primary, pressed:false, modifiers:egui::Modifiers::NONE }]);
            frame(ctx, app, vec![])
        }

        #[test]
        fn review_reverse_b_does_not_replay_the_old_rows_text() {
            let ctx=egui::Context::default();ctx.enable_accesskit();
            let (mut app,_)=fixture(true);
            let before=app.editor.project.clone();
            let output=frame(&ctx,&mut app,vec![]);
            let output=pointer_click_label(&ctx,&mut app,&output,"Edit group");
            let id=numeric(&output,0.9,0);
            frame(&ctx,&mut app,vec![action(id,Action::Focus)]);
            frame(&ctx,&mut app,vec![]);
            let output=frame(&ctx,&mut app,vec![key(egui::Key::A,egui::Modifiers::COMMAND),egui::Event::Text("0.831234567890123".into())]);
            let mut expected=app.sewing_edit.as_ref().unwrap().group.clone();
            reverse_sewing_side(&mut expected.b);
            pointer_click_label(&ctx,&mut app,&output,"Reverse B order and directions");
            let output=frame(&ctx,&mut app,vec![key(egui::Key::Tab,egui::Modifiers::NONE)]);
            eprintln!("Expected B[0] curve {} interval {} -> {}; actual {} -> {}",expected.b[0].curve,expected.b[0].t0,expected.b[0].t1,app.sewing_edit.as_ref().unwrap().group.b[0].t0,app.sewing_edit.as_ref().unwrap().group.b[0].t1);
            pointer_click_label(&ctx,&mut app,&output,"Apply group");
            assert!(app.sewing_edit.is_none(),"Unexpected validation rejection");
            assert_eq!(app.editor.project.revision,before.revision+1);
            stop_jobs(&mut app);
            assert_eq!(app.editor.project.sewing_groups[0],expected,"Apply persisted a replayed input onto an unrelated B span");
        }

