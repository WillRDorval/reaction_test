use rand::prelude::*;
use core::panic;
use std::{fs::OpenOptions, io::{BufWriter, Read, Write}, time::Instant};

use bevy::{prelude::*, winit::WinitSettings};

const SAMPLES: usize = 10;


fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .insert_resource(ReactSW(Instant::now()))
    .insert_resource(WinitSettings::desktop_app())
    .insert_resource(TrialStatus{
        atype: TrialType::Chroma,
        first: true
    })
    .insert_resource(User{
        sex: Sex::F,
        age: Age::Z,
        done: Stage::I
    })
    .insert_resource(CTrial{
        dir: Direction::F,
        time: 0
    })
    .add_event::<Sample>()
    .add_event::<TDone>()
    .add_systems(Startup, setup)
    .add_systems(Update, react)
    .add_systems(Update, sample)
    .add_systems(Update, finish)
    .run();
}

#[derive(Resource)]
struct ReactSW(Instant);

#[derive(PartialEq, Clone, Copy)]
enum TrialType{
    Chroma,
    Lightness
}

#[derive(Resource)]
struct TrialStatus{
    atype: TrialType,
    first: bool
}

#[derive(Clone, Copy)]
enum Direction{
    F, // -> R to G && B to W
    B  // -> G to R && W to B
}

#[derive(Resource)]
struct CTrial{
    dir: Direction,
    time: i128
}

#[derive(Clone, Copy)]
enum Sex{
    F = 1,
    M = 2,
    X = 3
}

#[derive(Clone, Copy)]
enum Age{
    Z = 1,
    Y = 2,
    X = 3,
    B = 4
}

enum Stage{
    I,
    S,
    A,
    T
}

#[derive(Resource)]
struct User{
    sex: Sex,
    age: Age,
    done: Stage
}

#[derive(Component)]
struct Trial{
    times: Vec<i32>,
    deltas: Vec<u32>,
    ttype: TrialType
}

#[derive(Event)]
struct Sample{
    stime: i32,
    sdelta: u32,
    stype: TrialType
}

#[derive(Event)]
struct TDone{
}



#[derive(Component)]
struct NodeChoice(u32);

fn react(key_in: Res<Input<KeyCode>>, 
    mut t: Query<(&mut Text, &NodeChoice)>, 
    mut user: ResMut<User>, 
    mut watch: ResMut<ReactSW>, 
    mut ctrial: ResMut<CTrial>, 
    mut bg: Query<(&mut BackgroundColor, &NodeChoice)>,
    stat: Res<TrialStatus>,
    mut ev_sample: EventWriter<Sample>,
    time: Res<Time>){
    match user.done{
        Stage::I => {
            if key_in.just_pressed(KeyCode::Space){
                for (mut text, tc) in &mut t {
                    if tc.0 == 1 {
                        text.sections[0].value = sex_string(user.sex);
                    }
                }
                user.done = Stage::S;
            }
        },
        Stage::S => {
            if key_in.just_pressed(KeyCode::Space){
                for (mut text, tc) in &mut t {
                    if tc.0 == 1 {
                        user.sex = next_sex(user.sex);
                        text.sections[0].value = sex_string(user.sex);
                    }
                }
            } else if key_in.just_pressed(KeyCode::Return){
                for (mut text, tc) in &mut t {
                    if tc.0 == 1 {
                        text.sections[0].value = age_string(user.age);
                    }
                }
                user.done = Stage::A;
            }
        },
        Stage::A => {
            if key_in.just_pressed(KeyCode::Space){
                for (mut text, tc) in &mut t {
                    if tc.0 == 1 {
                        user.age = next_age(user.age);
                        text.sections[0].value = age_string(user.age);
                    }
                }
            } else if key_in.just_pressed(KeyCode::Return){
                for (mut text, tc) in &mut t {
                    if tc.0 == 1 {
                        text.sections[0].value = "Press Space to Begin Trial\nPress Space as fast as you can when the screen changes colour".to_owned();
                    }
                }
                user.done = Stage::T;
            }
        },
        Stage::T => {
            match ctrial.time {
                0 => {
                    if key_in.just_pressed(KeyCode::Space){
                        ctrial.time = -1*rand::thread_rng().gen_range(1_500_00..3_000_00);
                        println!("Delay set to {} microseconds", ctrial.time);
                        watch.0 = Instant::now();
                        let mut cbg = bg.iter_mut().filter(|(_, tc)| tc.0==2).map(|(bg, _)| bg).next().unwrap();
                        cbg.0 = start_colour(ctrial.dir, stat.atype);
                        for (mut text, tc) in &mut t {
                            if tc.0 == 1 {
                                text.sections[0].value = "".to_owned();
                            }
                        }
                    }     
                },
                n if n > 0 => {
                    if key_in.just_pressed(KeyCode::Space){
                        
                        ev_sample.send(Sample{
                            stime: TryInto::<i32>::try_into(watch.0.elapsed().as_micros()).unwrap() * match ctrial.dir { Direction::F => 1, Direction::B => -1},
                            sdelta: time.delta().as_micros().try_into().unwrap(),
                            stype: stat.atype
                        });
                        ctrial.dir = match ctrial.dir {Direction::B => Direction::F, Direction::F => Direction::B};
                        ctrial.time = 0;
                        let mut cbg: Mut<'_, BackgroundColor> = bg.iter_mut().filter(|(_, tc)| tc.0==2).map(|(bg, _)| bg).next().unwrap();
                        cbg.0 = Color::BLACK;
                        for (mut text, tc) in &mut t {
                            if tc.0 == 1 {
                                text.sections[0].value = "Press Space when you're ready for the next trial".to_owned();
                            }
                        }
                    }
                },
                _ => {
                    for (mut text, tc) in &mut t {
                        if tc.0 == 1 {
                            text.sections[0].value = format!("{} us", watch.0.elapsed().as_micros()).to_owned();
                        }
                    }
                    if key_in.just_pressed(KeyCode::Space){
                        watch.0 = Instant::now();
                    }
                    if watch.0.elapsed().as_micros() >= (-1*ctrial.time).try_into().unwrap(){
                        let mut cbg: Mut<'_, BackgroundColor> = bg.iter_mut().filter(|(_, tc)| tc.0==2).map(|(bg, _)| bg).next().unwrap();
                        cbg.0 = end_colour(ctrial.dir, stat.atype);
                        ctrial.time = 1; 
                        watch.0 = Instant::now();
                    }
                    
                }
            }
        }
    }
}

fn next_age(age: Age) -> Age{
    match age {
        Age::Z => {
            Age::Y
        },
        Age::Y => {
            Age::X
        },
        Age::X => {
            Age::B
        },
        Age::B => {
            Age::Z
        }
    }
}

fn age_string(age: Age) -> String{
    match age {
        Age::Z => {
            "Select Your Age (Space to Cycle, Enter to select)\n18-27 <---\n28-43\n44-59\n60+".to_owned()
        },
        Age::Y => {
            "Select Your Age (Space to Cycle, Enter to select)\n18-27\n28-43 <---\n44-59\n60+".to_owned()
        },
        Age::X => {
            "Select Your Age (Space to Cycle, Enter to select)\n18-27\n28-43\n44-59 <---\n60+".to_owned()
        },
        Age::B => {
            "Select Your Age (Space to Cycle, Enter to select)\n18-27\n28-43\n44-59\n60+ <---".to_owned()
        }
    }
}

fn next_sex(sex: Sex) -> Sex {
    match sex {
        Sex::F => {
            Sex::M
        },
        Sex::M => {
            Sex::X
        },
        Sex::X => {
            Sex::F
        }
    }
}

fn sex_string(sex: Sex) -> String {
    match sex {
        Sex::F => {
            "Select Your Sex Assigned at Birth (Space to Cycle, Enter to select)\nF <---\nM\nX".to_owned()
        },
        Sex::M => {
            "Select Your Sex Assigned at Birth (Space to Cycle, Enter to select)\nF\nM <---\nX".to_owned()
        },
        Sex::X => {
            "Select Your Sex Assigned at Birth (Space to Cycle, Enter to select)\nF\nM\nX <---".to_owned()
        }
    }
}

fn start_colour(d: Direction, t: TrialType) -> Color{
    match (d, t) {
        (Direction::F, TrialType::Chroma) => {
            Color::RED
        },
        (Direction::F, TrialType::Lightness) => {
            Color::BLACK
        },
        (Direction::B, TrialType::Chroma) => {
            Color::GREEN
        },
        (Direction::B, TrialType::Lightness) => {
            Color::WHITE
        }
    }
}

fn end_colour(d: Direction, t: TrialType) -> Color{
    match (d, t) {
        (Direction::F, TrialType::Chroma) => {
            Color::GREEN
        },
        (Direction::F, TrialType::Lightness) => {
            Color::WHITE
        },
        (Direction::B, TrialType::Chroma) => {
            Color::RED
        },
        (Direction::B, TrialType::Lightness) => {
            Color::BLACK
        }
    }
}

fn sample(mut ev_sample: EventReader<Sample>, mut q: Query<&mut Trial>, mut ev_done: EventWriter<TDone>){
    for ev in ev_sample.read(){
        let mut t = q.iter_mut().filter(|e| e.ttype.eq(&ev.stype)).next().expect("One trial of each type IS instantiated");
        t.times.push(ev.stime);
        t.deltas.push(ev.sdelta);
        if t.times.len() >= SAMPLES{
            ev_done.send(TDone{});
        }
    }
}

fn finish(mut ev_done: EventReader<TDone>, u: Res<User>, mut stat: ResMut<TrialStatus>, q: Query<&mut Trial>){
    for _ in ev_done.read(){
        if stat.first{
            stat.atype = match stat.atype {
                TrialType::Chroma => TrialType::Lightness,
                TrialType::Lightness => TrialType::Chroma
            };
            stat.first = false;
        } else {
            let id_file = OpenOptions::new().truncate(true).read(true).write(true).create(true).open("./assets/results/id.txt");
            match id_file {
                Ok(mut f) => {
                    let mut buf: String = String::new();
                    f.read_to_string(&mut buf).expect("Error Reading ID file contents");
                    let id: u32 = match buf.lines().next(){
                        Some(n) => n.parse().expect("Corrupted ID file"),
                        None => 0
                    };
                    
                    f.write_all(format!("{}", id+1).as_bytes()).expect("Error writing ID file contents");
                    let data_file = OpenOptions::new().create_new(true).write(true).open(format!("./assets/results/data_{}.txt", id));
                    match data_file {
                        Ok(df ) => {
                            let ch_trial = q.iter().filter(|e| e.ttype == TrialType::Chroma).next().expect("One Chroma Trial exists");
                            let li_trial = q.iter().filter(|e| e.ttype == TrialType::Lightness).next().expect("One Lightness Trial exists");
                            let mut buf = BufWriter::new(df);
                            buf.write_all(format!("Age:{}, Sex:{}", u.age as u8, u.sex as u8).as_bytes()).expect("Error writing User Data Heading\n");
                            buf.write_all("CHROMA".as_bytes()).expect("Error Writing CHROMA heading");
                            for (data, unc) in ch_trial.times.iter().zip(ch_trial.deltas.iter()){
                                buf.write_all(format!("{}, {}", data, unc).as_bytes()).expect("Error Writing CHROMA trial data");
                            }

                            buf.write_all("LIGHTNESS".as_bytes()).expect("Error Writing LIGHTNESS heading");
                            for (data, unc) in ch_trial.times.iter().zip(li_trial.deltas.iter()){
                                buf.write_all(format!("{}, {}", data, unc).as_bytes()).expect("Error Writing LIGHTNESS trial data");
                            }
                        },
                        Err(_) => {
                            panic!("Couldn't open data file to write");
                        }
                    }
                },
                Err(_) => {
                    panic!("Couldn't open ID file dumbass");
                }
            }
        }
    }
}

fn setup(mut commands: Commands, mut tt: ResMut<TrialStatus>){
    commands.spawn(Camera2dBundle::default());
    commands.spawn((NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        background_color: BackgroundColor(Color::BLACK),
        ..default()
    }, NodeChoice(2))).with_children(|parent|{
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(25.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        }).with_children(|div| {
            div.spawn((
                    TextBundle::from_section("Press space to begin", TextStyle{
                        font_size: 40.0,
                        ..default()
                }),
                NodeChoice(1)
            ));
        });
    });
    commands.spawn(Trial{
        times: Vec::new(),
        deltas: Vec::new(),
        ttype: TrialType::Chroma
    });
    commands.spawn(Trial{
        times: Vec::new(),
        deltas: Vec::new(),
        ttype: TrialType::Lightness
    });

    if random(){
        tt.atype = TrialType::Lightness;
    }

}
